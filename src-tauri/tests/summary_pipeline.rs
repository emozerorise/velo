//! The summariser's transport, driven against a mock server.
//!
//! No API keys and no model are involved: these tests exist to pin the
//! behaviour that the design depends on -- that reasoning is switched off on
//! the Ollama path, that both dialects' streams are parsed, that a cancel
//! stops mid-stream, and that each failure class produces a message naming
//! its own fix.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use velo_lib::summary::transport::{self, ChatRequest, Ollama, OpenAi};

/// A one-shot HTTP server. Returns its base URL and a channel carrying the
/// request it received, so a test can assert on what was actually sent.
fn spawn_server(status: u16, frames: Vec<String>, gap: Duration) -> (String, Receiver<String>) {
    spawn_byte_server(
        status,
        frames.into_iter().map(String::into_bytes).collect(),
        gap,
    )
}

fn spawn_byte_server(
    status: u16,
    frames: Vec<Vec<u8>>,
    gap: Duration,
) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind");
    let port = listener.local_addr().expect("no local addr").port();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("no connection");
        stream.set_nodelay(true).expect("disable buffering");

        // Read the head, then however much body it declared.
        let mut data = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let read = stream.read(&mut buf).unwrap_or(0);
            if read == 0 {
                break;
            }
            data.extend_from_slice(&buf[..read]);

            let text = String::from_utf8_lossy(&data).to_string();
            if let Some(head_end) = text.find("\r\n\r\n") {
                let length = text
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("Content-Length: ")
                            .or_else(|| line.strip_prefix("content-length: "))
                    })
                    .and_then(|v| v.trim().parse::<usize>().ok())
                    .unwrap_or(0);

                if data.len() >= head_end + 4 + length {
                    break;
                }
            }
        }

        let _ = tx.send(String::from_utf8_lossy(&data).to_string());

        // No Content-Length: the body ends when the connection closes, which
        // is what lets frames be written with a gap between them.
        let head = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
            status,
            if status == 200 { "OK" } else { "Error" }
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.flush();

        for frame in frames {
            if stream.write_all(&frame).is_err() {
                return;
            }
            let _ = stream.flush();
            thread::sleep(gap);
        }
    });

    (format!("http://127.0.0.1:{}", port), rx)
}

fn request() -> ChatRequest {
    ChatRequest {
        model: "qwen3:8b".into(),
        system: "summarise".into(),
        user: "[00:01] hello".into(),
        context_tokens: 32_768,
        max_tokens: 2_048,
    }
}

fn ollama_frame(content: &str, done: bool) -> String {
    format!(
        "{{\"message\":{{\"content\":\"{}\"}},\"done\":{}}}\n",
        content, done
    )
}

#[tokio::test]
async fn summary_formats_reach_each_provider_in_single_map_and_reduce_passes() {
    use velo_lib::storage::settings_store::{SummaryFormat, SummarySettings};
    use velo_lib::summary::prompt;
    for provider in ["ollama", "openai", "deepseek"] {
        for format in [
            SummaryFormat::Meeting,
            SummaryFormat::KeyPoints,
            SummaryFormat::Custom,
        ] {
            let settings = SummarySettings {
                provider: provider.into(),
                format,
                language: "en".into(),
                instructions: "Focus on the central argument".into(),
                ..Default::default()
            };
            for (system, reminder, is_map) in [
                (
                    prompt::single_pass_system(&settings, "Velo"),
                    prompt::transcript_reminder(&settings),
                    false,
                ),
                (
                    prompt::map_system(&settings, "Velo"),
                    prompt::map_reminder(),
                    true,
                ),
                (
                    prompt::reduce_system(&settings, "Velo"),
                    prompt::notes_reminder(&settings),
                    false,
                ),
            ] {
                let frames = if provider == "ollama" {
                    vec![
                        ollama_frame("Synthetic answer [00:01]", false),
                        ollama_frame("", true),
                    ]
                } else {
                    vec!["data: {\"choices\":[{\"delta\":{\"content\":\"Synthetic answer [00:01]\"},\"finish_reason\":\"stop\"}]}\n\n".into(), "data: [DONE]\n\n".into()]
                };
                let (base, requests) = spawn_server(200, frames, Duration::ZERO);
                let answer = transport::stream_chat(
                    transport::dialect_for(provider).as_ref(),
                    &base,
                    None,
                    &ChatRequest {
                        system,
                        user: format!("[00:01] Synthetic evidence{reminder}"),
                        ..request()
                    },
                    &Arc::new(AtomicBool::new(false)),
                    |_| {},
                    || {},
                )
                .await
                .unwrap();
                assert_eq!(answer, "Synthetic answer [00:01]");
                let sent = requests.recv_timeout(Duration::from_secs(2)).unwrap();
                let body: serde_json::Value =
                    serde_json::from_str(sent.split_once("\r\n\r\n").unwrap().1).unwrap();
                let system = body["messages"][0]["content"].as_str().unwrap();
                let user = body["messages"][1]["content"].as_str().unwrap();
                assert!(system.contains("Focus on the central argument"));
                assert!(system.contains("Velo"));
                assert!(user.contains("[12:34]"));
                assert!(system.contains("A default setting is not an implementation order"));
                assert!(system.contains("same recording or the user's vocabulary confirms"));
                assert!(system.contains("Do not guess names or expand unclear fragments"));
                assert!(system.contains("Starting an action is not completing it"));
                assert!(system
                    .contains("Do not present an unclear phonetic token as verified terminology"));
                assert!(user.contains("Correct only confirmed spellings"));
                assert!(user.contains("preserve uncertainty and conditions"));
                if is_map {
                    assert!(user.contains("working notes, not the final summary"));
                    assert!(!user.contains("headings you were given"));
                } else {
                    match format {
                        SummaryFormat::Meeting => assert!(system.contains("## Decisions")),
                        SummaryFormat::KeyPoints => {
                            assert!(system.contains("## Key points"));
                            assert!(!system.contains("## Decisions"));
                        }
                        SummaryFormat::Custom => {
                            assert!(!system.contains("## Overview"));
                            assert!(!system.contains("exactly these five"));
                            assert!(user.contains("do not add preset headings"));
                        }
                    }
                }
            }
        }
    }
}

#[tokio::test]
async fn streams_an_ollama_answer_and_never_asks_it_to_think() {
    let (base, requests) = spawn_server(
        200,
        vec![
            ollama_frame("สรุป", false),
            ollama_frame(": เลื่อน deploy", false),
            ollama_frame("", true),
        ],
        Duration::from_millis(5),
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let mut deltas = Vec::new();

    let answer = transport::stream_chat(
        &Ollama,
        &base,
        None,
        &request(),
        &cancel,
        |text| deltas.push(text.to_string()),
        || {},
    )
    .await
    .expect("stream failed");

    assert_eq!(answer, "สรุป: เลื่อน deploy");
    assert_eq!(deltas.len(), 2, "each frame should arrive as its own delta");

    let sent = requests.recv().expect("no request captured");
    assert!(sent.contains("/api/chat"), "wrong route: {}", sent);
    assert!(
        sent.contains("\"think\":false"),
        "reasoning was not disabled: {}",
        sent
    );
    assert!(
        sent.contains("\"num_ctx\":32768"),
        "context window was not sent: {}",
        sent
    );
}

#[tokio::test]
async fn streams_an_openai_answer() {
    let (base, requests) = spawn_server(
        200,
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"one \"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"content\":\"two\"}}]}\n\n".into(),
            "data: [DONE]\n\n".into(),
        ],
        Duration::from_millis(5),
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let answer = transport::stream_chat(
        &OpenAi,
        &format!("{}/v1", base),
        None,
        &request(),
        &cancel,
        |_| {},
        || {},
    )
    .await
    .expect("stream failed");

    assert_eq!(answer, "one two");
    let sent = requests.recv().expect("no request captured");
    assert!(
        sent.contains("/v1/chat/completions"),
        "wrong route: {}",
        sent
    );
}

#[tokio::test]
async fn deepseek_streams_only_the_answer_with_a_key_and_thinking_disabled() {
    let (base, requests) = spawn_server(
        200,
        vec![
            ": keep-alive\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"private thoughts\"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"content\":\"สรุป [00:01]\"},\"finish_reason\":null}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n".into(),
            "data: [DONE]\n\n".into(),
        ],
        Duration::from_millis(5),
    );
    let dialect = transport::dialect_for("deepseek");
    let req = ChatRequest {
        model: "deepseek-flash".into(),
        max_tokens: 4_096,
        ..request()
    };
    let mut deltas = String::new();
    let mut thinking = 0;
    let answer = transport::stream_chat(
        dialect.as_ref(),
        &base,
        Some("sk-test-only"),
        &req,
        &AtomicBool::new(false),
        |text| deltas.push_str(text),
        || thinking += 1,
    )
    .await
    .expect("DeepSeek stream failed");
    assert_eq!(answer, "สรุป [00:01]");
    assert_eq!(deltas, "สรุป [00:01]");
    assert_eq!(thinking, 1);
    let sent = requests.recv().expect("request");
    assert!(sent.starts_with("POST /chat/completions "));
    assert!(sent
        .to_ascii_lowercase()
        .contains("authorization: bearer sk-test-only\r\n"));
    let body: serde_json::Value =
        serde_json::from_str(sent.split_once("\r\n\r\n").unwrap().1).unwrap();
    assert_eq!(body["thinking"]["type"], "disabled");
    assert_eq!(body["model"], "deepseek-flash");
    assert_eq!(body["max_tokens"], 4_096);
}

#[tokio::test]
async fn deepseek_lists_models_with_bearer_auth() {
    let (base, requests) = spawn_server(
        200,
        vec![r#"{"object":"list","data":[{"id":"deepseek-flash","object":"model","owned_by":"deepseek"},{"id":"deepseek-v4-pro","object":"model","owned_by":"deepseek"}]}"#.into()],
        Duration::ZERO,
    );
    let dialect = transport::dialect_for("deepseek");
    let names = transport::list_models(dialect.as_ref(), &base, Some("sk-test-only"))
        .await
        .expect("DeepSeek probe");
    assert_eq!(names, vec!["deepseek-flash", "deepseek-v4-pro"]);
    let sent = requests.recv().expect("request");
    assert!(sent.starts_with("GET /models "));
    assert!(sent
        .to_ascii_lowercase()
        .contains("authorization: bearer sk-test-only\r\n"));
}

#[tokio::test]
async fn deepseek_keeps_thai_characters_split_across_network_reads() {
    let frame = "data: {\"choices\":[{\"delta\":{\"content\":\"สรุป [00:01]\"}}]}\n\n";
    let split = frame.find('ส').unwrap() + 1;
    let (base, _) = spawn_byte_server(
        200,
        vec![
            frame.as_bytes()[..split].to_vec(),
            frame.as_bytes()[split..].to_vec(),
            b"data: [DONE]\n\n".to_vec(),
        ],
        Duration::from_millis(50),
    );
    let dialect = transport::dialect_for("deepseek");
    let answer = transport::stream_chat(
        dialect.as_ref(),
        &base,
        None,
        &request(),
        &AtomicBool::new(false),
        |_| {},
        || {},
    )
    .await
    .expect("split UTF-8 stream");
    assert_eq!(answer, "สรุป [00:01]");
}

#[tokio::test]
async fn deepseek_does_not_treat_a_token_limit_as_a_complete_summary() {
    let (base, _) = spawn_server(
        200,
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"unfinished\"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"length\"}]}\n\n".into(),
            "data: [DONE]\n\n".into(),
        ],
        Duration::from_millis(5),
    );
    let dialect = transport::dialect_for("deepseek");
    let mut partial = String::new();
    let error = transport::stream_chat(
        dialect.as_ref(),
        &base,
        None,
        &request(),
        &AtomicBool::new(false),
        |text| partial.push_str(text),
        || {},
    )
    .await
    .expect_err("a truncated answer must not be saved as complete");
    assert!(error.to_string().contains("output limit"));
    assert_eq!(partial, "unfinished");
}

#[tokio::test]
async fn deepseek_does_not_accept_an_empty_answer_after_done() {
    let (base, _) = spawn_server(
        200,
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"   \"}}]}\n\n".into(),
            "data: [DONE]\n\n".into(),
        ],
        Duration::ZERO,
    );
    let dialect = transport::dialect_for("deepseek");
    let error = transport::stream_chat(
        dialect.as_ref(),
        &base,
        None,
        &request(),
        &AtomicBool::new(false),
        |_| {},
        || {},
    )
    .await
    .expect_err("an empty summary must not be saved");
    assert!(error.to_string().contains("returned nothing"));
}

#[tokio::test]
async fn deepseek_reports_in_stream_failures_instead_of_saving_partial_answers() {
    for (frame, expected) in [
        (
            r#"{"choices":[{"delta":{},"finish_reason":"content_filter"}]}"#,
            "content filter",
        ),
        (
            r#"{"error":{"message":"provider unavailable","type":"server_error","code":"server_error"}}"#,
            "reported an error",
        ),
        (
            r#"{"choices":[{"delta":{},"finish_reason":"insufficient_system_resource"}]}"#,
            "interrupted",
        ),
        (
            r#"{"choices":[{"delta":{},"finish_reason":"aborted"}]}"#,
            "interrupted",
        ),
        (
            r#"{"choices":[{"delta":{},"finish_reason":"tool_calls"}]}"#,
            "interrupted",
        ),
    ] {
        let (base, _) = spawn_server(
            200,
            vec![
                "data: {\"choices\":[{\"delta\":{\"content\":\"unfinished\"}}]}\n\n".into(),
                format!("data: {frame}\n\n"),
                "data: [DONE]\n\n".into(),
            ],
            Duration::ZERO,
        );
        let dialect = transport::dialect_for("deepseek");
        let error = transport::stream_chat(
            dialect.as_ref(),
            &base,
            None,
            &request(),
            &AtomicBool::new(false),
            |_| {},
            || {},
        )
        .await
        .expect_err("a provider failure must not be saved as a completed summary");
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[tokio::test]
async fn strips_inline_reasoning_from_the_openai_dialect() {
    // Servers other than Ollama put reasoning in the content itself.
    let (base, _requests) = spawn_server(
        200,
        vec![
            "data: {\"choices\":[{\"delta\":{\"content\":\"<think>weigh\"}}]}\n\n".into(),
            "data: {\"choices\":[{\"delta\":{\"content\":\" it up</think>answer\"}}]}\n\n".into(),
            "data: [DONE]\n\n".into(),
        ],
        Duration::from_millis(5),
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let answer = transport::stream_chat(
        &OpenAi,
        &format!("{}/v1", base),
        None,
        &request(),
        &cancel,
        |_| {},
        || {},
    )
    .await
    .expect("stream failed");

    assert_eq!(answer, "answer");
}

#[tokio::test]
async fn cancelling_stops_mid_stream() {
    let (base, _requests) = spawn_server(
        200,
        vec![
            ollama_frame("first", false),
            ollama_frame("second", false),
            ollama_frame("third", false),
            ollama_frame("", true),
        ],
        // Long enough that the cancel lands before the rest arrives.
        Duration::from_millis(200),
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let mut seen = 0;

    let result = transport::stream_chat(
        &Ollama,
        &base,
        None,
        &request(),
        &cancel,
        |_| {
            seen += 1;
            flag.store(true, Ordering::Relaxed);
        },
        || {},
    )
    .await;

    assert!(
        result.is_err(),
        "a cancelled stream must not return an answer"
    );
    assert!(result.unwrap_err().to_string().contains("Cancelled"));
    assert_eq!(seen, 1, "should have stopped after the first frame");
}

#[tokio::test]
async fn a_missing_model_names_itself() {
    let (base, _requests) = spawn_server(
        404,
        vec!["{\"error\":\"model not found\"}".into()],
        Duration::ZERO,
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let error = transport::stream_chat(&Ollama, &base, None, &request(), &cancel, |_| {}, || {})
        .await
        .expect_err("expected a failure");

    let message = error.to_string();
    assert!(
        message.contains("qwen3:8b"),
        "unhelpful message: {}",
        message
    );
}

#[tokio::test]
async fn a_rejected_key_says_so() {
    let (base, _requests) =
        spawn_server(401, vec!["{\"error\":\"bad key\"}".into()], Duration::ZERO);

    let cancel = Arc::new(AtomicBool::new(false));
    let error = transport::stream_chat(
        &OpenAi,
        &format!("{}/v1", base),
        Some("sk-not-a-real-key"),
        &request(),
        &cancel,
        |_| {},
        || {},
    )
    .await
    .expect_err("expected a failure");

    assert!(error.to_string().contains("API key was rejected"));
}

#[tokio::test]
async fn an_unreachable_server_is_reported_as_such() {
    // Nothing is listening on this port.
    let cancel = Arc::new(AtomicBool::new(false));
    let error = transport::stream_chat(
        &Ollama,
        "http://127.0.0.1:1",
        None,
        &request(),
        &cancel,
        |_| {},
        || {},
    )
    .await
    .expect_err("expected a failure");

    assert!(error.to_string().contains("Could not reach"));
}

#[tokio::test]
async fn listing_models_reads_both_dialects() {
    let (base, _requests) = spawn_server(
        200,
        vec!["{\"models\":[{\"name\":\"qwen3:8b\"},{\"name\":\"llama3:8b\"}]}".into()],
        Duration::ZERO,
    );
    let names = transport::list_models(&Ollama, &base, None)
        .await
        .expect("probe failed");
    assert_eq!(names, vec!["qwen3:8b", "llama3:8b"]);

    let (base, _requests) = spawn_server(
        200,
        vec!["{\"data\":[{\"id\":\"gpt-4o-mini\"}]}".into()],
        Duration::ZERO,
    );
    let names = transport::list_models(&OpenAi, &format!("{}/v1", base), None)
        .await
        .expect("probe failed");
    assert_eq!(names, vec!["gpt-4o-mini"]);
}

/// The whole local path against a real server, skipped unless asked for.
/// Point `VELO_TEST_TRANSCRIPT` at a cached transcript to run a real meeting
/// through it rather than the five-line sample:
///
/// ```sh
/// VELO_SUMMARY_LIVE=1 cargo test --test summary_pipeline -- --nocapture live
/// ```
#[tokio::test]
async fn live_ollama_writes_a_thai_summary() {
    if std::env::var("VELO_SUMMARY_LIVE").is_err() {
        eprintln!("skipping: set VELO_SUMMARY_LIVE=1 to run against a real server");
        return;
    }

    let base =
        std::env::var("VELO_SUMMARY_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into());
    let model = std::env::var("VELO_SUMMARY_MODEL").unwrap_or_else(|_| "qwen3:30b-instruct".into());

    let sample = [
        (0.0, "สวัสดีครับ วันนี้คุยเรื่องกำหนดปล่อยรุ่นหน้า"),
        (12.0, "สมชาย: ขอเลื่อน deploy จากพุธไปศุกร์ เพราะ QA ยังไม่จบ"),
        (20.0, "ฝน: ได้ค่ะ เดี๋ยวแจ้งทีม QA แล้วอัปเดตในบอร์ดให้"),
        (35.0, "สมชาย: อีกเรื่องคือ dashboard ยังโหลดช้า ขอให้ดูสัปดาห์หน้า"),
        (48.0, "ฝน: ยังไม่แน่ใจว่าจะได้งบเพิ่มไหม ขอถามหัวหน้าก่อน"),
    ]
    .iter()
    .map(|(start, text)| velo_lib::transcript::TranscriptSegment {
        start: *start,
        end: start + 10.0,
        text: (*text).into(),
    })
    .collect::<Vec<_>>();

    // A real meeting, when one is offered.
    let (segments, detected) = match std::env::var("VELO_TEST_TRANSCRIPT") {
        Ok(path) => {
            let json = std::fs::read_to_string(&path).expect("could not read the transcript");
            let transcript: velo_lib::transcript::Transcript =
                serde_json::from_str(&json).expect("that is not a transcript");
            eprintln!(
                "using {} segments ({}) from {}",
                transcript.segments.len(),
                transcript.language,
                path
            );
            (transcript.segments, transcript.language)
        }
        Err(_) => (sample, "th".to_string()),
    };

    // Exactly what the app does: the setting is left on "auto", and the
    // language whisper reported decides what the prompt asks for.
    let settings = velo_lib::storage::settings_store::SummarySettings {
        model: model.clone(),
        language: velo_lib::summary::prompt::resolve_language("auto", &detected),
        ..Default::default()
    };
    assert_eq!(
        settings.language, "th",
        "auto did not resolve to the recording"
    );

    let chunks = velo_lib::summary::chunk::chunk(
        &segments,
        velo_lib::summary::chunk::budget_bytes(settings.context_tokens),
    );
    eprintln!(
        "{} chars over {} chunk(s)",
        chunks.iter().map(|c| c.text.len()).sum::<usize>(),
        chunks.len()
    );

    let cancel = Arc::new(AtomicBool::new(false));
    let answer = transport::stream_chat(
        &Ollama,
        &base,
        None,
        &ChatRequest {
            model,
            system: velo_lib::summary::prompt::single_pass_system(
                &settings,
                "deploy, QA, dashboard",
            ),
            user: format!(
                "{}{}",
                chunks[0].text,
                velo_lib::summary::prompt::transcript_reminder(&settings)
            ),
            context_tokens: settings.context_tokens,
            max_tokens: 2_048,
        },
        &cancel,
        |_| {},
        || {},
    )
    .await
    .expect("the model did not answer");

    println!("\n----- summary -----\n{}\n-------------------", answer);

    assert!(!answer.trim().is_empty());
    assert!(answer.contains('#'), "no headings in the answer");
    assert!(
        answer.contains("ภาพรวม"),
        "a Thai recording came back with English headings"
    );
    assert!(
        answer.contains(":") && answer.contains('['),
        "the model dropped the timestamps"
    );
    assert!(
        !answer.contains("<think>"),
        "reasoning leaked into the answer"
    );
}
