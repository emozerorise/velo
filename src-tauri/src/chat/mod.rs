//! File-scoped discussion of owned source snapshots; storage and credentials stay in commands.
pub mod context;
pub mod prompt;
pub mod types;

use crate::errors::{ModelErrorCode, VeloError};
use crate::jobs::{JobCancel, JobLease};
use crate::storage::settings_store::SummarySettings;
use crate::summary::transport::{self, ConversationRequest};
use crate::transcript::TranscriptSegment;
use parking_lot::Mutex;
use std::sync::{atomic::Ordering, Arc};
use types::*;

pub struct ChatInput {
    pub path: String,
    pub request_id: String,
    pub settings: SummarySettings,
    pub segments: Vec<TranscriptSegment>,
    pub summary: Option<String>,
    pub history: Vec<ChatHistoryMessage>,
    pub message: String,
    pub language: String,
    pub api_key: Option<String>,
}

pub fn validate_id(id: &str) -> ChatResult<()> {
    if id.is_empty() || id.len() > 128 || !id.is_ascii() || id.bytes().any(|b| b.is_ascii_control())
    {
        Err(ChatFailure::new(ChatErrorCode::InvalidInput))
    } else {
        Ok(())
    }
}

fn prepare(input: &ChatInput) -> ChatResult<ChatContext> {
    validate_id(&input.request_id)?;
    if input.path.is_empty() {
        return Err(ChatFailure::new(ChatErrorCode::InvalidInput));
    }
    if input.segments.is_empty() || input.segments.iter().all(|s| s.text.trim().is_empty()) {
        return Err(ChatFailure::new(ChatErrorCode::MissingTranscript));
    }
    if !valid_base_url(&input.settings.base_url)
        || input.settings.model.trim().is_empty()
        || !matches!(
            input.settings.provider.as_str(),
            "ollama" | "openai" | "deepseek"
        )
    {
        return Err(ChatFailure::new(ChatErrorCode::NotConfigured));
    }
    if input.settings.provider == "deepseek"
        && input
            .api_key
            .as_ref()
            .is_none_or(|key| key.trim().is_empty())
    {
        return Err(ChatFailure::new(ChatErrorCode::Auth));
    }
    context::validate(&input.segments, &input.history, &input.message)
        .map_err(|_| ChatFailure::new(ChatErrorCode::InvalidInput))?;
    context::build_context(
        &input.segments,
        input.summary.as_deref(),
        &input.history,
        &input.message,
        input.settings.context_tokens,
        &input.language,
    )
    .map_err(|_| ChatFailure::new(ChatErrorCode::Context))
}

pub fn valid_base_url(base: &str) -> bool {
    reqwest::Url::parse(base).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
            && url.query().is_none()
            && url.fragment().is_none()
    })
}

fn transport_failure(error: VeloError) -> ChatFailure {
    ChatFailure::new(match error {
        VeloError::Model { code, .. } => match code {
            ModelErrorCode::Auth => ChatErrorCode::Auth,
            ModelErrorCode::Connection => ChatErrorCode::Connection,
            ModelErrorCode::RateLimit => ChatErrorCode::RateLimit,
            ModelErrorCode::Provider => ChatErrorCode::Provider,
            ModelErrorCode::Context => ChatErrorCode::Context,
            ModelErrorCode::Timeout => ChatErrorCode::Timeout,
            ModelErrorCode::Empty => ChatErrorCode::Empty,
            ModelErrorCode::Incomplete | ModelErrorCode::OutputLimit => ChatErrorCode::Incomplete,
            ModelErrorCode::Cancelled => ChatErrorCode::Cancelled,
        },
        _ => ChatErrorCode::Provider,
    })
}

fn emit_failure(input: &ChatInput, error: &ChatFailure, emit: &dyn Fn(ChatEvent)) {
    emit(ChatEvent::Error(ChatError {
        path: input.path.clone(),
        request_id: input.request_id.clone(),
        code: error.code,
        message: error.message.clone(),
        cancelled: error.code == ChatErrorCode::Cancelled,
    }));
}

pub async fn run_turn(
    input: ChatInput,
    cancel: JobCancel,
    emit: Arc<dyn Fn(ChatEvent) + Send + Sync>,
) -> ChatResult<ChatAnswer> {
    match prepare(&input) {
        Ok(context) => run_prepared(input, context, cancel, emit).await,
        Err(error) => {
            emit_failure(&input, &error, emit.as_ref());
            Err(error)
        }
    }
}

async fn run_prepared(
    input: ChatInput,
    context: ChatContext,
    cancel: JobCancel,
    emit: Arc<dyn Fn(ChatEvent) + Send + Sync>,
) -> ChatResult<ChatAnswer> {
    let dialect = transport::dialect_for(&input.settings.provider);
    let request = ConversationRequest {
        model: input.settings.model.clone(),
        messages: context.messages,
        context_tokens: input.settings.context_tokens,
        max_tokens: 2048,
    };
    let mut sequence = 0;
    let result = transport::stream_conversation(
        dialect.as_ref(),
        &input.settings.base_url,
        input.api_key.as_deref(),
        &request,
        &cancel,
        |text| {
            sequence += 1;
            emit(ChatEvent::Delta(ChatDelta {
                path: input.path.clone(),
                request_id: input.request_id.clone(),
                sequence,
                text: text.into(),
            }));
        },
        || {},
    )
    .await
    .map_err(transport_failure);
    let result = if cancel.flag().load(Ordering::Acquire) {
        Err(ChatFailure::new(ChatErrorCode::Cancelled))
    } else {
        result
    };
    match result {
        Ok(text) => {
            let answer = ChatAnswer {
                path: input.path,
                request_id: input.request_id,
                text,
                sources: context.sources,
                coverage: context.coverage,
            };
            emit(ChatEvent::Ready(answer.clone()));
            Ok(answer)
        }
        Err(error) => {
            emit_failure(&input, &error, emit.as_ref());
            Err(error)
        }
    }
}

#[derive(Default, Clone)]
pub struct ChatState {
    active: Arc<Mutex<Option<ActiveTurn>>>,
}
struct ActiveTurn {
    id: String,
    token: Arc<()>,
    cancel: JobCancel,
}

struct Cleanup {
    state: ChatState,
    token: Arc<()>,
}

pub struct ChatReservation {
    id: String,
    cancel: JobCancel,
    cleanup: Cleanup,
}

impl ChatReservation {
    pub fn start(
        self,
        input: ChatInput,
        lease: JobLease,
        emit: Arc<dyn Fn(ChatEvent) + Send + Sync>,
    ) -> ChatResult<ChatStarted> {
        if input.request_id != self.id {
            return Err(ChatFailure::new(ChatErrorCode::InvalidInput));
        }
        let context = prepare(&input)?;
        if self.cancel.flag().load(Ordering::Acquire) {
            return Err(ChatFailure::new(ChatErrorCode::Cancelled));
        }
        let started = ChatStarted {
            path: input.path.clone(),
            request_id: input.request_id.clone(),
            sources: context.sources.clone(),
            coverage: context.coverage.clone(),
        };
        tokio::spawn(async move {
            let _lease = lease;
            let _cleanup = self.cleanup;
            let _ = run_prepared(input, context, self.cancel, emit).await;
        });
        Ok(started)
    }
}
impl Drop for Cleanup {
    fn drop(&mut self) {
        let mut active = self.state.active.lock();
        if active
            .as_ref()
            .is_some_and(|a| Arc::ptr_eq(&a.token, &self.token))
        {
            *active = None;
        }
    }
}

impl ChatState {
    pub fn start(
        &self,
        input: ChatInput,
        lease: JobLease,
        emit: Arc<dyn Fn(ChatEvent) + Send + Sync>,
    ) -> ChatResult<ChatStarted> {
        self.reserve(&input.request_id)?.start(input, lease, emit)
    }

    pub fn reserve(&self, request_id: &str) -> ChatResult<ChatReservation> {
        validate_id(request_id)?;
        let mut active = self.active.lock();
        if active.is_some() {
            return Err(ChatFailure::new(ChatErrorCode::Busy));
        }
        let cancel = JobCancel::new();
        let token = Arc::new(());
        *active = Some(ActiveTurn {
            id: request_id.to_string(),
            token: token.clone(),
            cancel: cancel.clone(),
        });
        drop(active);
        let cleanup = Cleanup {
            state: self.clone(),
            token,
        };
        Ok(ChatReservation {
            id: request_id.into(),
            cancel,
            cleanup,
        })
    }
    pub fn cancel(&self, request_id: &str) {
        if let Some(active) = self.active.lock().as_ref().filter(|a| a.id == request_id) {
            active.cancel.cancel();
        }
    }
    pub fn cancel_all(&self) {
        if let Some(active) = self.active.lock().as_ref() {
            active.cancel.cancel();
        }
    }
    pub fn active_request(&self) -> Option<String> {
        self.active.lock().as_ref().map(|a| a.id.clone())
    }
}
