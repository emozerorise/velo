//! Synthetic loopback transport and resource-lifetime regressions. No model or keychain.
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;
use velo_lib::jobs::JobCancel;
use velo_lib::summary::transport::{
    self, ConversationRequest, DeepSeek, Ollama, OpenAi, WireMessage, WireRole,
};

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
    let cancellation = async move {
        ready.await.unwrap();
        if !headers {
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
                cancel.cancel();
            }
        },
        || {},
    );
    let (result, ()) = tokio::time::timeout(Duration::from_millis(250), async {
        tokio::join!(run, cancellation)
    })
    .await
    .expect("Stop must interrupt the request independently of provider timeout");
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
