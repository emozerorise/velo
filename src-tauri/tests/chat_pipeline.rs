//! Synthetic loopback transport and resource-lifetime regressions. No model or keychain.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};
use velo_lib::chat::types::{ChatErrorCode, ChatEvent};
use velo_lib::chat::{run_turn, ChatInput, ChatState};
use velo_lib::commands::chat::{canonical_path, load_snapshot, status_from_snapshot};
use velo_lib::jobs::JobCancel;
use velo_lib::storage::settings_store::SummarySettings;
use velo_lib::summary::transport::{
    self, ConversationRequest, DeepSeek, Ollama, OpenAi, WireMessage, WireRole,
};
use velo_lib::summary::{Summary, SummaryStore};
use velo_lib::transcript::{Transcript, TranscriptSegment, TranscriptStore};

mod jobs {
    use velo_lib::jobs::{JobGate, JobResource};

    #[test]
    fn model_lease_excludes_summary_and_chat() {
        let gate = JobGate::default();
        let summary = gate.try_acquire(JobResource::ChatModel, "summary").unwrap();
        assert!(gate.try_acquire(JobResource::ChatModel, "chat").is_err());
        drop(summary);
        assert!(gate.try_acquire(JobResource::ChatModel, "chat").is_ok());
    }

    #[test]
    fn failed_spawn_releases_own_lease() {
        let gate = JobGate::default();
        let lease = gate
            .try_acquire(JobResource::ChatModel, "failed-spawn")
            .unwrap();
        let rejected: Result<(), _> = Err(lease);
        drop(rejected);
        let newer = gate.try_acquire(JobResource::ChatModel, "newer").unwrap();
        assert!(gate.try_acquire(JobResource::ChatModel, "third").is_err());
        drop(newer);
        assert!(gate.try_acquire(JobResource::ChatModel, "third").is_ok());
    }

    #[test]
    fn whisper_lease_excludes_file_and_dictation() {
        let gate = JobGate::default();
        let file = gate.try_acquire(JobResource::Whisper, "file").unwrap();
        assert!(gate.try_acquire(JobResource::Whisper, "dictation").is_err());
        assert!(gate.try_acquire(JobResource::ChatModel, "chat").is_ok());
        drop(file);
        assert!(gate.try_acquire(JobResource::Whisper, "dictation").is_ok());
    }
}

fn read_request(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let mut request = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let read = stream.read(&mut chunk).unwrap_or(0);
        if read == 0 {
            break;
        }
        request.extend_from_slice(&chunk[..read]);
        let text = String::from_utf8_lossy(&request);
        if let Some(end) = text.find("\r\n\r\n") {
            let length = text
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .and_then(|n| n.trim().parse::<usize>().ok())
                })
                .unwrap_or(0);
            if request.len() >= end + 4 + length {
                break;
            }
        }
    }
    String::from_utf8(request).unwrap()
}

fn request() -> ConversationRequest {
    ConversationRequest {
        model: "synthetic".into(),
        context_tokens: 8192,
        max_tokens: 2048,
        messages: vec![
            WireMessage {
                role: WireRole::System,
                content: "Ground answers in sources".into(),
            },
            WireMessage {
                role: WireRole::User,
                content: "คำถาม".into(),
            },
            WireMessage {
                role: WireRole::Assistant,
                content: "คำตอบเดิม".into(),
            },
            WireMessage {
                role: WireRole::User,
                content: "ต่อไป?".into(),
            },
        ],
    }
}

fn server(response: String) -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let sent = read_request(&mut stream);
        let _ = tx.send(sent);
        let _ = stream.write_all(response.as_bytes());
    });
    (base, rx)
}

fn ok(body: &str) -> String {
    format!("HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{body}")
}

async fn cancelled_stall(headers: bool) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (accepted, ready) = tokio::sync::oneshot::channel();
    let (release, wait) = mpsc::channel::<()>();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        // Slow setup is not Stop latency: the budget starts at cancellation.
        thread::sleep(Duration::from_millis(300));
        if headers {
            stream
                .write_all(
                    ok("{\"message\":{\"content\":\"partial\"},\"done\":false}\n").as_bytes(),
                )
                .unwrap();
        }
        let _ = accepted.send(());
        let _ = wait.recv_timeout(Duration::from_secs(2));
    });
    let cancel = JobCancel::new();
    let signal = cancel.clone();
    let (cancelled_at, timing) = mpsc::channel();
    let before_headers = cancelled_at.clone();
    let cancellation = async move {
        ready.await.unwrap();
        if !headers {
            before_headers.send(Instant::now()).unwrap();
            signal.cancel();
        }
    };
    let req = request();
    let run = transport::stream_conversation(
        &Ollama,
        &base,
        None,
        &req,
        &cancel,
        |_| {
            if headers {
                cancelled_at.send(Instant::now()).unwrap();
                cancel.cancel();
            }
        },
        || {},
    );
    let (result, ()) = tokio::time::timeout(Duration::from_secs(5), async {
        tokio::join!(run, cancellation)
    })
    .await
    .expect("The mock connection and cancellation must finish within the setup watchdog");
    assert!(
        timing.try_recv().unwrap().elapsed() < Duration::from_millis(250),
        "Stop must interrupt the request within 250ms of cancellation"
    );
    release.send(()).unwrap();
    assert!(result.unwrap_err().to_string().contains("Cancelled"));
}

#[tokio::test]
async fn cancel_before_headers_finishes_within_250ms() {
    cancelled_stall(false).await;
}

#[tokio::test]
async fn cancel_between_frames_finishes_within_250ms() {
    cancelled_stall(true).await;
}

#[tokio::test]
async fn redirect_does_not_forward_bearer() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let second = TcpListener::bind("127.0.0.1:0").unwrap();
    second.set_nonblocking(true).unwrap();
    let target = format!("http://{}", second.local_addr().unwrap());
    let count = Arc::new(AtomicUsize::new(0));
    let received = count.clone();
    let (stop, wait) = mpsc::channel::<()>();
    let receiver = thread::spawn(move || loop {
        if wait.try_recv().is_ok() {
            break;
        }
        if let Ok((mut stream, _)) = second.accept() {
            received.fetch_add(1, Ordering::SeqCst);
            read_request(&mut stream);
            let _ = stream.write_all(ok("data: [DONE]\n").as_bytes());
        }
        thread::sleep(Duration::from_millis(1));
    });
    let (base, _) = server(format!(
        "HTTP/1.1 307 Redirect\r\nLocation: {target}\r\nContent-Length: 0\r\n\r\n"
    ));
    let result = transport::stream_conversation(
        &OpenAi,
        &base,
        Some("synthetic-secret"),
        &request(),
        &JobCancel::new(),
        |_| {},
        || {},
    )
    .await;
    stop.send(()).unwrap();
    receiver.join().unwrap();
    assert!(result.is_err());
    assert_eq!(count.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn error_body_is_not_user_visible() {
    for probe in [false, true] {
        let (base, _) = server("HTTP/1.1 401 Unauthorized\r\nConnection: close\r\n\r\nsynthetic-secret private transcript".into());
        let result = if probe {
            transport::list_models(&OpenAi, &base, Some("synthetic-secret"))
                .await
                .map(|_| String::new())
        } else {
            transport::stream_conversation(
                &OpenAi,
                &base,
                Some("synthetic-secret"),
                &request(),
                &JobCancel::new(),
                |_| {},
                || {},
            )
            .await
        };
        let error = result.unwrap_err().to_string();
        assert!(error.contains("API key was rejected"));
        assert!(!error.contains("synthetic-secret"));
        assert!(!error.contains("private transcript"));
    }
}

#[tokio::test]
async fn terminal_frame_keeps_its_final_text() {
    for (dialect, body) in [
        (Box::new(Ollama) as Box<dyn transport::ChatDialect>, "{\"message\":{\"content\":\"ท้าย\"},\"done\":true}\n"),
        (Box::new(OpenAi), "data: {\"choices\":[{\"delta\":{\"content\":\"ท้าย\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n"),
        (Box::new(DeepSeek), "data: {\"choices\":[{\"delta\":{\"content\":\"ท้าย\"},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n"),
    ] {
        let (base, sent) = server(ok(body));
        let mut deltas = Vec::new();
        let answer = transport::stream_conversation(dialect.as_ref(), &base, None, &request(), &JobCancel::new(), |s| deltas.push(s.to_string()), || {}).await.unwrap();
        assert_eq!(answer, "ท้าย"); assert_eq!(deltas.concat(), answer);
        let raw = sent.recv().unwrap();
        let body: serde_json::Value = serde_json::from_str(raw.split("\r\n\r\n").nth(1).unwrap()).unwrap();
        assert_eq!(body["messages"].as_array().unwrap().len(), 4);
        assert_eq!(body["messages"][2]["role"], "assistant");
        if dialect.chat_url("").contains("api/chat") { assert_eq!(body["think"], false); }
    }
}

#[tokio::test]
async fn chat_rejects_silent_eof_and_bounded_output() {
    for body in [
        "{\"message\":{\"content\":\"partial\"},\"done\":false}\n".to_string(),
        "x".repeat(262145),
        format!(
            "{{\"message\":{{\"content\":\"{}\"}},\"done\":true}}\n",
            "x".repeat(65537)
        ),
    ] {
        let (base, _) = server(ok(&body));
        assert!(transport::stream_conversation(
            &Ollama,
            &base,
            None,
            &request(),
            &JobCancel::new(),
            |_| {},
            || {}
        )
        .await
        .is_err());
    }
}

fn input(base: &str, provider: &str, id: &str) -> ChatInput {
    ChatInput {
        path: "/synthetic.wav".into(),
        request_id: id.into(),
        settings: SummarySettings {
            provider: provider.into(),
            base_url: base.into(),
            model: "synthetic".into(),
            context_tokens: 8192,
            ..SummarySettings::default()
        },
        segments: vec![TranscriptSegment {
            start: 0.0,
            end: 9.0,
            text: "A synthetic decision".into(),
        }],
        summary: None,
        history: vec![],
        message: "Next?".into(),
        language: "en".into(),
        api_key: (provider == "deepseek").then(|| "synthetic-key".into()),
    }
}

#[tokio::test]
async fn runner_uses_each_dialect_and_emits_matching_grounding_metadata() {
    for (provider, frame) in [
        ("ollama", "{\"message\":{\"content\":\"answer [00:00]\"},\"done\":true}\n"),
        ("openai", "data: {\"choices\":[{\"delta\":{\"content\":\"answer [00:00]\"},\"finish_reason\":\"stop\"}]}\n"),
        ("deepseek", "data: {\"choices\":[{\"delta\":{\"content\":\"answer [00:00]\"},\"finish_reason\":\"stop\"}]}\n"),
    ] {
        let (base, received) = server(ok(frame));
        let events = Arc::new(std::sync::Mutex::new(Vec::new()));
        let observed = events.clone();
        let answer = run_turn(input(&base, provider, "turn"), JobCancel::new(), Arc::new(move |event| observed.lock().unwrap().push(event))).await.unwrap();
        assert_eq!(answer.sources[0].start, 0.0);
        let events = events.lock().unwrap();
        assert!(events.iter().any(|e| matches!(e, ChatEvent::Ready(ready) if ready.sources == answer.sources && ready.text == answer.text)));
        assert!(events.iter().any(|e| matches!(e, ChatEvent::Delta(delta) if delta.sequence == 1 && delta.request_id == "turn")));
        let sent = received.recv().unwrap();
        assert!(!sent.contains("base64")); assert!(!sent.contains("audio_url"));
        if provider == "deepseek" { assert!(sent.contains("\"thinking\":{\"type\":\"disabled\"}")); }
    }
}

#[tokio::test]
async fn no_transcript_rejects_before_http() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut turn = input(
        &format!("http://{}", listener.local_addr().unwrap()),
        "ollama",
        "empty",
    );
    turn.segments.clear();
    let error = run_turn(turn, JobCancel::new(), Arc::new(|_| {}))
        .await
        .unwrap_err();
    assert_eq!(error.code, ChatErrorCode::MissingTranscript);
    assert!(listener.accept().is_err());
}

#[tokio::test]
async fn switch_cancel_drops_old_turn() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (accepted, ready) = tokio::sync::oneshot::channel();
    let (release, wait) = mpsc::channel::<()>();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        let _ = accepted.send(());
        let _ = wait.recv_timeout(Duration::from_secs(2));
    });
    let cancel = JobCancel::new();
    let signal = cancel.clone();
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let captured = events.clone();
    let result = tokio::join!(
        run_turn(
            input(&base, "ollama", "old"),
            cancel,
            Arc::new(move |e| captured.lock().unwrap().push(e))
        ),
        async move {
            ready.await.unwrap();
            signal.cancel();
        }
    )
    .0;
    release.send(()).unwrap();
    assert_eq!(result.unwrap_err().code, ChatErrorCode::Cancelled);
    assert!(!events
        .lock()
        .unwrap()
        .iter()
        .any(|e| matches!(e, ChatEvent::Ready(_))));
}

#[tokio::test]
async fn late_completion_cannot_release_new_job() {
    use velo_lib::jobs::{JobGate, JobResource};
    let gate = JobGate::default();
    let state = ChatState::default();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (accepted, ready) = tokio::sync::oneshot::channel();
    let (release, wait) = mpsc::channel::<()>();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        read_request(&mut stream);
        let _ = accepted.send(());
        let _ = wait.recv_timeout(Duration::from_secs(2));
    });
    let (terminal, mut event) = tokio::sync::mpsc::unbounded_channel();
    state
        .start(
            input(&base, "ollama", "old"),
            gate.try_acquire(JobResource::ChatModel, "old").unwrap(),
            Arc::new(move |e| {
                let _ = terminal.send(e);
            }),
        )
        .unwrap();
    ready.await.unwrap();
    assert!(state
        .start(
            input(&base, "ollama", "old"),
            JobGate::default()
                .try_acquire(JobResource::ChatModel, "duplicate")
                .unwrap(),
            Arc::new(|_| {})
        )
        .is_err());
    state.cancel("old");
    tokio::time::timeout(Duration::from_millis(250), event.recv())
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(Duration::from_millis(250), async {
        while state.active_request().is_some() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    release.send(()).unwrap();
    let lease = gate.try_acquire(JobResource::ChatModel, "new").unwrap();
    state.cancel("old");
    assert!(gate.try_acquire(JobResource::ChatModel, "third").is_err());
    drop(lease);
}

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        Self::with_stamp(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
        )
    }
    fn with_stamp(stamp: u128) -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "velo-chat-{}-{}-{}",
            std::process::id(),
            stamp,
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn media(&self) -> String {
        self.0.join("media.wav").to_str().unwrap().to_string()
    }
}

#[test]
fn fixtures_with_the_same_clock_stamp_do_not_collide() {
    let first = Fixture::with_stamp(123);
    let second = Fixture::with_stamp(123);
    assert_ne!(first.0, second.0);
    assert!(first.0.is_dir() && second.0.is_dir());
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn canonical_aliases_share_identity() {
    let root = Fixture::new();
    std::fs::write(root.media(), b"synthetic").unwrap();
    let alias = root.0.join(".").join("media.wav");
    assert_eq!(
        canonical_path(&root.media()).unwrap(),
        canonical_path(alias.to_str().unwrap()).unwrap()
    );
    let error = canonical_path("/not/a/private/existing/path").unwrap_err();
    assert!(!error.message.contains("/not/a/private"));
}

#[test]
fn status_does_not_probe() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let settings = SummarySettings {
        base_url: format!("http://{}", listener.local_addr().unwrap()),
        ..SummarySettings::default()
    };
    let status = status_from_snapshot("/canonical.wav".into(), &settings, true, false);
    assert!(status.has_transcript && status.configured);
    assert!(!status.has_key);
    assert!(listener.accept().is_err());
}

#[test]
fn summary_without_cloud_key_remains_readable() {
    let root = Fixture::new();
    let store = SummaryStore::open(root.0.join("summaries")).unwrap();
    let summary = Summary {
        path: root.media(),
        model: "synthetic".into(),
        language: "en".into(),
        markdown: "Cached facts".into(),
        created_at: 0,
        source_segments: 1,
    };
    store.save(&summary).unwrap();
    let settings = SummarySettings {
        provider: "deepseek".into(),
        base_url: "https://api.deepseek.com".into(),
        model: "synthetic".into(),
        ..SummarySettings::default()
    };
    let status = status_from_snapshot(summary.path.clone(), &settings, true, false);
    assert!(!status.has_key);
    assert_eq!(store.load(&summary.path).unwrap().markdown, "Cached facts");
}

#[tokio::test]
async fn chat_never_writes_summary_or_transcript_cache() {
    let root = Fixture::new();
    std::fs::write(root.media(), b"synthetic").unwrap();
    let transcripts = TranscriptStore::open(root.0.join("transcripts")).unwrap();
    let summaries = SummaryStore::open(root.0.join("summaries")).unwrap();
    let alias = root
        .0
        .join(".")
        .join("media.wav")
        .to_str()
        .unwrap()
        .to_string();
    transcripts
        .save(&Transcript {
            path: alias.clone(),
            language: "en".into(),
            model: "synthetic".into(),
            prompt: "".into(),
            created_at: 0,
            segments: input("", "ollama", "").segments,
        })
        .unwrap();
    summaries
        .save(&Summary {
            path: alias,
            model: "synthetic".into(),
            language: "en".into(),
            markdown: "Cached".into(),
            created_at: 0,
            source_segments: 1,
        })
        .unwrap();
    let snapshot = || {
        ["transcripts", "summaries"]
            .into_iter()
            .flat_map(|dir| {
                std::fs::read_dir(root.0.join(dir))
                    .unwrap()
                    .map(|entry| {
                        let file = entry.unwrap().path();
                        (file.clone(), std::fs::read(file).unwrap())
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let before = snapshot();
    let (path, transcript, summary) =
        load_snapshot(&root.media(), &transcripts, &summaries).unwrap();
    let (base, _) = server(ok("{\"message\":{\"content\":\"Answer\"},\"done\":true}\n"));
    let mut turn = input(&base, "ollama", "cache");
    turn.path = path;
    turn.segments = transcript.segments;
    turn.summary = summary.map(|s| s.markdown);
    run_turn(turn, JobCancel::new(), Arc::new(|_| {}))
        .await
        .unwrap();
    assert_eq!(before, snapshot());
}

#[tokio::test]
async fn runner_errors_have_stable_codes_without_provider_body() {
    for (status, code) in [
        (401, ChatErrorCode::Auth),
        (429, ChatErrorCode::RateLimit),
        (500, ChatErrorCode::Provider),
    ] {
        let (base, _) = server(format!(
            "HTTP/1.1 {status} Error\r\nConnection: close\r\n\r\nprivate source synthetic-secret"
        ));
        let error = run_turn(
            input(&base, "ollama", "error"),
            JobCancel::new(),
            Arc::new(|_| {}),
        )
        .await
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(!error.message.contains("private"));
        assert!(!error.message.contains("secret"));
    }
}

#[tokio::test]
async fn cancelled_reservation_never_starts_http_after_preflight() {
    use velo_lib::jobs::{JobGate, JobResource};
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let state = ChatState::default();
    let gate = JobGate::default();
    let pending = state.reserve("pending").unwrap();
    state.cancel("pending");
    let error = pending
        .start(
            input(&base, "ollama", "pending"),
            gate.try_acquire(JobResource::ChatModel, "pending").unwrap(),
            Arc::new(|_| {}),
        )
        .unwrap_err();
    assert_eq!(error.code, ChatErrorCode::Cancelled);
    assert!(listener.accept().is_err());
    assert!(state.active_request().is_none());
    assert!(gate.try_acquire(JobResource::ChatModel, "next").is_ok());
}

#[test]
fn invalid_destination_cannot_expose_url_credentials() {
    for base in [
        "data:text/plain,synthetic-secret",
        "https://user:synthetic-secret@example.com/v1?key=synthetic-secret",
    ] {
        let settings = SummarySettings {
            base_url: base.into(),
            ..SummarySettings::default()
        };
        let status = status_from_snapshot("/canonical.wav".into(), &settings, true, false);
        assert!(!status.configured);
        assert!(!status.base_url.contains("synthetic-secret"));
    }
}

#[test]
fn canonical_cache_miss_uses_latest_alias_without_migrating_files() {
    let root = Fixture::new();
    std::fs::write(root.media(), b"synthetic").unwrap();
    std::fs::create_dir(root.0.join("nested")).unwrap();
    let store = TranscriptStore::open(root.0.join("transcripts")).unwrap();
    let summaries = SummaryStore::open(root.0.join("summaries")).unwrap();
    for (alias, created_at, text) in [
        (root.0.join(".").join("media.wav"), 1, "old"),
        (
            root.0.join("nested").join("..").join("media.wav"),
            2,
            "latest",
        ),
    ] {
        let path = alias.to_str().unwrap().to_string();
        store
            .save(&Transcript {
                path: path.clone(),
                language: "en".into(),
                model: "synthetic".into(),
                prompt: "".into(),
                created_at,
                segments: vec![TranscriptSegment {
                    start: 0.0,
                    end: 1.0,
                    text: text.into(),
                }],
            })
            .unwrap();
        summaries
            .save(&Summary {
                path,
                language: "en".into(),
                model: "synthetic".into(),
                created_at,
                source_segments: 1,
                markdown: text.into(),
            })
            .unwrap();
    }
    let (_, transcript, summary) = load_snapshot(&root.media(), &store, &summaries).unwrap();
    assert_eq!(transcript.segments[0].text, "latest");
    assert_eq!(summary.unwrap().markdown, "latest");
    assert_eq!(
        std::fs::read_dir(root.0.join("transcripts"))
            .unwrap()
            .count(),
        2
    );
}
