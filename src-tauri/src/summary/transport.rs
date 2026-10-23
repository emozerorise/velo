//! Talking to a chat model.
//!
//! Provider dialects are not interchangeable. Ollama's
//! native route is the local path because it is the only one that can switch
//! qwen3's reasoning off -- measured at 22 generated tokens against 409 for
//! the same summary through the OpenAI-compatible route -- and the only one
//! that can set the context window. The OpenAI dialect is what every other
//! server speaks. DeepSeek adds an explicit non-thinking mode to that wire
//! format without sending its vendor-specific fields to other servers.
//!
//! The dialect trait is deliberately synchronous: it only builds URLs and
//! bodies and reads single frames, so the streaming machinery below is
//! written once and shared.

use crate::errors::{ModelErrorCode, Result, VeloError};
use crate::jobs::JobCancel;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Long enough for a slow first token on a cold model, short enough that a
/// wedged server does not hang the panel forever. `max_tokens` bounds the
/// real runtime; this only catches a server that has stopped answering.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(600);
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

/// Low, because a meeting summary that invents details is worse than a dull
/// one.
const TEMPERATURE: f64 = 0.3;

/// qwen3:8b ships with `repeat_penalty 1` -- no penalty at all -- and on a
/// long Thai transcript it will restate the same points until something
/// stops it. Measured: one 34-minute meeting ran to 9,000 tokens without
/// finishing. The llama.cpp default puts an end to that.
const REPEAT_PENALTY: f64 = 1.1;

pub struct ChatRequest {
    pub model: String,
    pub system: String,
    pub user: String,
    pub context_tokens: u32,
    /// Hard ceiling on the answer. A summary that has not finished by here
    /// is repeating itself, and cutting it off beats waiting out a loop.
    pub max_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WireRole {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireMessage {
    pub role: WireRole,
    pub content: String,
}

pub struct ConversationRequest {
    pub model: String,
    pub messages: Vec<WireMessage>,
    pub context_tokens: u32,
    pub max_tokens: u32,
}

/// One frame of a streamed response.
pub enum Frame {
    /// Carries nothing (a keep-alive, or a blank line between SSE events).
    Empty,
    Text(String),
    /// The model is reasoning; there is no answer text yet.
    Thinking,
    Done,
    DoneText(String),
    Failed(&'static str),
    OutputLimit(&'static str),
}

pub trait ChatDialect: Send + Sync {
    fn chat_url(&self, base: &str) -> String;
    fn models_url(&self, base: &str) -> String;
    fn body(&self, req: &ChatRequest) -> Value;
    fn frame(&self, line: &str) -> Frame;
    fn models(&self, value: &Value) -> Vec<String>;
    /// True when this dialect's servers may inline `<think>` blocks in the
    /// answer instead of reporting reasoning separately.
    fn inlines_reasoning(&self) -> bool;
}

pub fn dialect_for(provider: &str) -> Box<dyn ChatDialect> {
    match provider {
        "openai" => Box::new(OpenAi),
        "deepseek" => Box::new(DeepSeek),
        _ => Box::new(Ollama),
    }
}

fn trim_base(base: &str) -> &str {
    base.trim_end_matches('/')
}

// ---------------------------------------------------------------------------
// Ollama native
// ---------------------------------------------------------------------------

pub struct Ollama;

impl ChatDialect for Ollama {
    fn chat_url(&self, base: &str) -> String {
        format!("{}/api/chat", trim_base(base))
    }

    fn models_url(&self, base: &str) -> String {
        format!("{}/api/tags", trim_base(base))
    }

    fn body(&self, req: &ChatRequest) -> Value {
        json!({
            "model": req.model,
            "stream": true,
            // The whole reason this dialect exists. Without it qwen3 spends
            // most of its output budget thinking out loud.
            "think": false,
            "options": {
                "num_ctx": req.context_tokens,
                "num_predict": req.max_tokens,
                "repeat_penalty": REPEAT_PENALTY,
                "temperature": TEMPERATURE,
            },
            "messages": [
                { "role": "system", "content": req.system },
                { "role": "user", "content": req.user },
            ],
        })
    }

    fn frame(&self, line: &str) -> Frame {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            return Frame::Empty;
        };

        let message = &value["message"];
        if value["done"].as_bool().unwrap_or(false) {
            if value["done_reason"].as_str() == Some("length") {
                return Frame::OutputLimit("The model reached the output limit");
            }
            return match message["content"].as_str() {
                Some(text) if !text.is_empty() => Frame::DoneText(text.to_string()),
                _ => Frame::Done,
            };
        }
        match message["content"].as_str() {
            Some(text) if !text.is_empty() => Frame::Text(text.to_string()),
            _ => {
                if message["thinking"].as_str().is_some_and(|t| !t.is_empty()) {
                    Frame::Thinking
                } else {
                    Frame::Empty
                }
            }
        }
    }

    fn models(&self, value: &Value) -> Vec<String> {
        value["models"]
            .as_array()
            .map(|models| {
                models
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn inlines_reasoning(&self) -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible
// ---------------------------------------------------------------------------

/// `base_url` includes any version segment the server requires, for example
/// `https://api.openai.com/v1` (DeepSeek's documented root has none).
pub struct OpenAi;

impl ChatDialect for OpenAi {
    fn chat_url(&self, base: &str) -> String {
        format!("{}/chat/completions", trim_base(base))
    }

    fn models_url(&self, base: &str) -> String {
        format!("{}/models", trim_base(base))
    }

    fn body(&self, req: &ChatRequest) -> Value {
        // No context field: this dialect has nowhere to put one, which is
        // why chunks are sized against the setting instead.
        json!({
            "model": req.model,
            "stream": true,
            "max_tokens": req.max_tokens,
            "temperature": TEMPERATURE,
            "messages": [
                { "role": "system", "content": req.system },
                { "role": "user", "content": req.user },
            ],
        })
    }

    fn frame(&self, line: &str) -> Frame {
        let Some(payload) = line.strip_prefix("data:") else {
            return Frame::Empty;
        };
        let payload = payload.trim();

        if payload == "[DONE]" {
            return Frame::Done;
        }

        let Ok(value) = serde_json::from_str::<Value>(payload) else {
            return Frame::Empty;
        };

        let delta = &value["choices"][0]["delta"];
        match value["choices"][0]["finish_reason"].as_str() {
            Some("length") => return Frame::OutputLimit("The provider reached the output limit"),
            Some("stop") => {
                return match delta["content"].as_str() {
                    Some(text) if !text.is_empty() => Frame::DoneText(text.to_string()),
                    _ => Frame::Done,
                }
            }
            Some(_) => {
                return Frame::Failed("The provider interrupted the answer before completion")
            }
            None => {}
        }
        match delta["content"].as_str() {
            Some(text) if !text.is_empty() => Frame::Text(text.to_string()),
            _ => {
                if delta["reasoning"].as_str().is_some_and(|t| !t.is_empty()) {
                    Frame::Thinking
                } else {
                    Frame::Empty
                }
            }
        }
    }

    fn models(&self, value: &Value) -> Vec<String> {
        value["data"]
            .as_array()
            .map(|models| {
                models
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn inlines_reasoning(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// DeepSeek (OpenAI wire format, non-thinking summaries)
// ---------------------------------------------------------------------------

pub struct DeepSeek;

impl ChatDialect for DeepSeek {
    fn chat_url(&self, base: &str) -> String {
        OpenAi.chat_url(base)
    }

    fn models_url(&self, base: &str) -> String {
        OpenAi.models_url(base)
    }

    fn body(&self, req: &ChatRequest) -> Value {
        let mut body = OpenAi.body(req);
        // DeepSeek enables thinking by default. Summaries need the answer,
        // not a reasoning pass that spends the output budget before it.
        body["thinking"] = json!({ "type": "disabled" });
        body
    }

    fn frame(&self, line: &str) -> Frame {
        let value = line
            .strip_prefix("data:")
            .and_then(|payload| serde_json::from_str::<Value>(payload.trim()).ok());
        if let Some(value) = value.as_ref() {
            match value["choices"][0]["finish_reason"].as_str() {
                Some("length") => return Frame::OutputLimit(
                    "DeepSeek reached the output limit. The partial summary was not saved; try a more concise summary in Extra instructions."
                ),
                Some("content_filter") => return Frame::Failed(
                    "DeepSeek could not complete this summary because of its content filter."
                ),
                None | Some("stop") => {},
                Some(_) => return Frame::Failed(
                    "DeepSeek interrupted the summary before it was complete. Please try again."
                ),
            }
            if value.get("error").is_some() {
                return Frame::Failed(
                    "DeepSeek reported an error while writing the summary. Please try again.",
                );
            }
        }
        let frame = OpenAi.frame(line);
        if matches!(frame, Frame::Empty) {
            if let Some(value) = value {
                if value["choices"][0]["delta"]["reasoning_content"]
                    .as_str()
                    .is_some_and(|text| !text.is_empty())
                {
                    return Frame::Thinking;
                }
            }
        }
        frame
    }

    fn models(&self, value: &Value) -> Vec<String> {
        OpenAi.models(value)
    }

    fn inlines_reasoning(&self) -> bool {
        true
    }
}

// ---------------------------------------------------------------------------
// Inline reasoning
// ---------------------------------------------------------------------------

/// Removes `<think>...</think>` from a stream that may split a tag across
/// two deltas. Servers of the OpenAI dialect other than Ollama do inline
/// reasoning this way, and it must never reach the reader.
#[derive(Default)]
pub struct ThinkStripper {
    inside: bool,
    /// A partial `<think` or `</think` still waiting for the rest of itself.
    pending: String,
}

impl ThinkStripper {
    const OPEN: &'static str = "<think>";
    const CLOSE: &'static str = "</think>";

    pub fn push(&mut self, text: &str) -> String {
        self.pending.push_str(text);
        let mut out = String::new();

        loop {
            let tag = if self.inside { Self::CLOSE } else { Self::OPEN };

            if let Some(at) = self.pending.find(tag) {
                if !self.inside {
                    out.push_str(&self.pending[..at]);
                }
                self.pending = self.pending[at + tag.len()..].to_string();
                self.inside = !self.inside;
                continue;
            }

            // Hold back anything that could still turn into the tag we are
            // looking for once more text arrives.
            let keep = (1..tag.len())
                .rev()
                .find(|n| self.pending.len() >= *n && self.pending.ends_with(&tag[..*n]))
                .unwrap_or(0);

            let split = self.pending.len() - keep;
            if !self.inside {
                out.push_str(&self.pending[..split]);
            }
            self.pending = self.pending[split..].to_string();
            return out;
        }
    }
}

// ---------------------------------------------------------------------------
// Streaming
// ---------------------------------------------------------------------------

fn client(timeout: Duration) -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| VeloError::Summary("Could not create an HTTP client".into()))
}

/// Turns a failed response into a message that names the fix, since every one
/// of these has a different one.
fn http_error(status: reqwest::StatusCode, model: &str) -> VeloError {
    let message = match status.as_u16() {
        401 | 403 => "The API key was rejected".to_string(),
        404 => format!("The model \"{}\" is not available on this server", model),
        429 => "The provider is rate limiting this request".to_string(),
        500..=599 => "The provider failed to answer".to_string(),
        _ => format!("The provider returned {}", status),
    };

    model_error(
        match status.as_u16() {
            401 | 403 => ModelErrorCode::Auth,
            429 => ModelErrorCode::RateLimit,
            _ => ModelErrorCode::Provider,
        },
        message,
    )
}

fn model_error(code: ModelErrorCode, message: impl Into<String>) -> VeloError {
    VeloError::Model {
        code,
        message: message.into(),
    }
}
fn legacy_error(error: VeloError) -> VeloError {
    // Summary recovery needs the category, not a match against vendor wording.
    match error {
        error @ VeloError::Model {
            code: ModelErrorCode::OutputLimit,
            ..
        } => error,
        _ => VeloError::Summary(error.to_string()),
    }
}

fn transport_error(e: reqwest::Error) -> VeloError {
    if e.is_connect() {
        model_error(
            ModelErrorCode::Connection,
            "Could not reach the model server",
        )
    } else if e.is_timeout() {
        model_error(
            ModelErrorCode::Timeout,
            "The model server took too long to answer",
        )
    } else {
        model_error(ModelErrorCode::Provider, "The model request failed")
    }
}

fn completed_answer(answer: String) -> Result<String> {
    if answer.trim().is_empty() {
        Err(model_error(
            ModelErrorCode::Empty,
            "The model returned nothing. It may have run out of context.",
        ))
    } else {
        Ok(answer)
    }
}

/// Ask the server what it has. Used before a long job starts, so a stopped
/// server is found in seconds rather than after an hour of transcription.
pub async fn list_models(
    dialect: &dyn ChatDialect,
    base_url: &str,
    api_key: Option<&str>,
) -> Result<Vec<String>> {
    list_models_inner(dialect, base_url, api_key)
        .await
        .map_err(legacy_error)
}

async fn list_models_inner(
    dialect: &dyn ChatDialect,
    base_url: &str,
    api_key: Option<&str>,
) -> Result<Vec<String>> {
    let mut request = client(PROBE_TIMEOUT)?.get(dialect.models_url(base_url));
    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = request.send().await.map_err(transport_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(http_error(status, ""));
    }

    let value: Value = response
        .json()
        .await
        .map_err(|_| VeloError::Summary("The server sent an unreadable list".into()))?;

    Ok(dialect.models(&value))
}

/// Stream one completion, handing answer text to `on_text` as it arrives and
/// signalling `on_thinking` while the model has not started answering.
///
/// Returns the full answer. Cancellation stops mid-stream and drops the
/// response, which closes the connection.
pub async fn stream_chat(
    dialect: &dyn ChatDialect,
    base_url: &str,
    api_key: Option<&str>,
    req: &ChatRequest,
    cancel: &AtomicBool,
    on_text: impl FnMut(&str),
    on_thinking: impl FnMut(),
) -> Result<String> {
    stream_body(
        dialect,
        base_url,
        api_key,
        dialect.body(req),
        &req.model,
        CancelSignal::Legacy(cancel),
        true,
        on_text,
        on_thinking,
    )
    .await
    .map_err(legacy_error)
}

pub async fn stream_conversation(
    dialect: &dyn ChatDialect,
    base_url: &str,
    api_key: Option<&str>,
    req: &ConversationRequest,
    cancel: &JobCancel,
    on_text: impl FnMut(&str),
    on_thinking: impl FnMut(),
) -> Result<String> {
    let mut body = dialect.body(&ChatRequest {
        model: req.model.clone(),
        system: String::new(),
        user: String::new(),
        context_tokens: req.context_tokens,
        max_tokens: req.max_tokens,
    });
    body["messages"] = serde_json::to_value(&req.messages)
        .map_err(|_| VeloError::Summary("Invalid conversation".into()))?;
    stream_body(
        dialect,
        base_url,
        api_key,
        body,
        &req.model,
        CancelSignal::Conversation(cancel),
        true,
        on_text,
        on_thinking,
    )
    .await
}

enum CancelSignal<'a> {
    Legacy(&'a AtomicBool),
    Conversation(&'a JobCancel),
}

impl CancelSignal<'_> {
    fn is_cancelled(&self) -> bool {
        match self {
            Self::Legacy(flag) => flag.load(Ordering::Acquire),
            Self::Conversation(signal) => signal.flag().load(Ordering::Acquire),
        }
    }
    async fn cancelled(&self) {
        match self {
            Self::Conversation(signal) => signal.cancelled().await,
            Self::Legacy(flag) => {
                while !flag.load(Ordering::Acquire) {
                    tokio::time::sleep(Duration::from_millis(20)).await;
                }
            }
        }
    }
}

fn cancelled_error() -> VeloError {
    model_error(ModelErrorCode::Cancelled, "Cancelled")
}

#[allow(clippy::too_many_arguments)]
async fn stream_body(
    dialect: &dyn ChatDialect,
    base_url: &str,
    api_key: Option<&str>,
    body: Value,
    model: &str,
    cancel: CancelSignal<'_>,
    strict: bool,
    mut on_text: impl FnMut(&str),
    mut on_thinking: impl FnMut(),
) -> Result<String> {
    let mut request = client(REQUEST_TIMEOUT)?
        .post(dialect.chat_url(base_url))
        .json(&body);
    if let Some(key) = api_key {
        request = request.bearer_auth(key);
    }

    let response = tokio::select! {
        biased;
        _ = cancel.cancelled() => return Err(cancelled_error()),
        response = request.send() => response.map_err(transport_error)?,
    };
    let status = response.status();
    if !status.is_success() {
        return Err(http_error(status, model));
    }

    let mut stripper = dialect.inlines_reasoning().then(ThinkStripper::default);
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();
    let mut answer = String::new();

    loop {
        let next = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(cancelled_error()),
            next = stream.next() => next,
        };
        let eof = next.is_none();
        let bytes = next
            .transpose()
            .map_err(transport_error)?
            .unwrap_or_default();
        // Split bytes before decoding: a packet can contain many lines, or
        // end inside a Thai character. Bounds apply to one accumulated line.
        let pieces = bytes
            .split_inclusive(|b| *b == b'\n')
            .chain(eof.then_some(&[][..]));
        for piece in pieces {
            if cancel.is_cancelled() {
                return Err(cancelled_error());
            }
            if strict && buffer.len().saturating_add(piece.len()) > 256 * 1024 {
                return Err(model_error(
                    ModelErrorCode::Context,
                    "The provider stream exceeded the line limit",
                ));
            }
            buffer.extend_from_slice(piece);
            if !eof && !piece.ends_with(b"\n") {
                continue;
            }
            let line = std::str::from_utf8(&buffer).map_err(|_| {
                model_error(ModelErrorCode::Provider, "The provider sent invalid UTF-8")
            })?;
            let frame = dialect.frame(line.trim());
            buffer.clear();
            let terminal = matches!(frame, Frame::Done | Frame::DoneText(_));
            match frame {
                Frame::Empty => {}
                Frame::Thinking => on_thinking(),
                Frame::Done => {}
                Frame::Failed(message) => {
                    return Err(model_error(ModelErrorCode::Provider, message))
                }
                Frame::OutputLimit(message) => {
                    return Err(model_error(ModelErrorCode::OutputLimit, message))
                }
                Frame::Text(text) | Frame::DoneText(text) => {
                    let text = match stripper.as_mut() {
                        Some(stripper) => stripper.push(&text),
                        None => text,
                    };
                    if !text.is_empty() {
                        if strict && answer.len().saturating_add(text.len()) > 64 * 1024 {
                            return Err(model_error(
                                ModelErrorCode::Context,
                                "The provider answer exceeded the output limit",
                            ));
                        }
                        answer.push_str(&text);
                        on_text(&text);
                    }
                }
            }
            if terminal {
                return completed_answer(answer);
            }
        }
        if eof {
            break;
        }
    }
    if strict {
        Err(model_error(
            ModelErrorCode::Incomplete,
            "The provider stream ended before completion",
        ))
    } else {
        completed_answer(answer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text_of(frame: Frame) -> Option<String> {
        match frame {
            Frame::Text(t) => Some(t),
            _ => None,
        }
    }

    #[test]
    fn ollama_always_disables_thinking() {
        let body = Ollama.body(&ChatRequest {
            model: "qwen3:8b".into(),
            system: "s".into(),
            user: "u".into(),
            context_tokens: 32_768,
            max_tokens: 2_048,
        });

        assert_eq!(body["think"], json!(false));
        assert_eq!(body["options"]["num_ctx"], json!(32_768));
        assert_eq!(body["stream"], json!(true));
        // Both halves of the runaway fix: a ceiling, and a reason to stop.
        assert_eq!(body["options"]["num_predict"], json!(2_048));
        assert_eq!(body["options"]["repeat_penalty"], json!(1.1));
    }

    #[test]
    fn ollama_reads_its_own_frames() {
        let content = r#"{"message":{"content":"สรุป"},"done":false}"#;
        assert_eq!(text_of(Ollama.frame(content)).as_deref(), Some("สรุป"));

        let thinking = r#"{"message":{"content":"","thinking":"hmm"},"done":false}"#;
        assert!(matches!(Ollama.frame(thinking), Frame::Thinking));

        let done = r#"{"message":{"content":""},"done":true}"#;
        assert!(matches!(Ollama.frame(done), Frame::Done));
    }

    #[test]
    fn openai_reads_sse_frames() {
        let content = r#"data: {"choices":[{"delta":{"content":"hi"}}]}"#;
        assert_eq!(text_of(OpenAi.frame(content)).as_deref(), Some("hi"));

        let reasoning = r#"data: {"choices":[{"delta":{"content":"","reasoning":"why"}}]}"#;
        assert!(matches!(OpenAi.frame(reasoning), Frame::Thinking));

        assert!(matches!(OpenAi.frame("data: [DONE]"), Frame::Done));
        assert!(matches!(OpenAi.frame(": keep-alive"), Frame::Empty));
    }

    #[test]
    fn deepseek_disables_thinking_without_changing_openai_requests() {
        let req = ChatRequest {
            model: "deepseek-flash".into(),
            system: "summarise in Thai".into(),
            user: "[00:01] hello".into(),
            context_tokens: 32_768,
            max_tokens: 4_096,
        };
        let body = dialect_for("deepseek").body(&req);
        assert_eq!(body["thinking"]["type"], "disabled");
        assert_eq!(body["max_tokens"], 4_096);
        assert_eq!(body["temperature"], 0.3);
        assert_eq!(body["stream"], true);
        assert_eq!(body["messages"][0]["content"], "summarise in Thai");
        assert_eq!(body["messages"][1]["content"], "[00:01] hello");
        assert!(OpenAi.body(&req).get("thinking").is_none());
    }

    #[test]
    fn deepseek_uses_openai_routes_and_separates_reasoning_from_answers() {
        let dialect = dialect_for("deepseek");
        assert_eq!(
            dialect.chat_url("https://api.deepseek.com/"),
            "https://api.deepseek.com/chat/completions"
        );
        assert_eq!(
            dialect.models_url("https://api.deepseek.com/"),
            "https://api.deepseek.com/models"
        );
        assert!(matches!(
            dialect
                .frame(r#"data: {"choices":[{"delta":{"reasoning_content":"private thoughts"}}]}"#),
            Frame::Thinking
        ));
        assert_eq!(
            text_of(dialect.frame(r#"data: {"choices":[{"delta":{"content":"สรุป [00:01]"}}]}"#)),
            Some("สรุป [00:01]".into())
        );
    }

    #[test]
    fn urls_survive_a_trailing_slash() {
        assert_eq!(
            Ollama.chat_url("http://localhost:11434/"),
            "http://localhost:11434/api/chat"
        );
        assert_eq!(
            OpenAi.chat_url("https://api.openai.com/v1"),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn strips_a_whole_think_block() {
        let mut stripper = ThinkStripper::default();
        let out = stripper.push("<think>reasoning here</think>answer");
        assert_eq!(out, "answer");
    }

    #[test]
    fn strips_a_tag_split_across_deltas() {
        let mut stripper = ThinkStripper::default();
        let mut out = String::new();

        // The opening tag arrives in three pieces, the closing one in two.
        for piece in ["before", "<th", "ink>hidden", " more</thi", "nk>after"] {
            out.push_str(&stripper.push(piece));
        }

        assert_eq!(out, "beforeafter");
    }

    #[test]
    fn passes_plain_text_through_untouched() {
        let mut stripper = ThinkStripper::default();
        let out = stripper.push("just an answer with < and > in it");
        assert_eq!(out, "just an answer with < and > in it");
    }
}
