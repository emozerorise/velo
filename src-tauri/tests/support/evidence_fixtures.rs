//! Synthetic HTTP responses only; no recording, keychain or semantic oracle.
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::mpsc,
    thread,
    time::Duration,
};
pub fn server(answers: Vec<serde_json::Value>) -> (String, mpsc::Receiver<serde_json::Value>) {
    let (base, requests, _, _) = serve(answers, None);
    (base, requests)
}
pub fn server_gated(
    answers: Vec<serde_json::Value>,
    pause_request: usize,
) -> (
    String,
    mpsc::Receiver<serde_json::Value>,
    tokio::sync::oneshot::Receiver<()>,
    mpsc::Sender<()>,
) {
    serve(answers, Some(pause_request))
}
fn serve(
    answers: Vec<serde_json::Value>,
    pause_request: Option<usize>,
) -> (
    String,
    mpsc::Receiver<serde_json::Value>,
    tokio::sync::oneshot::Receiver<()>,
    mpsc::Sender<()>,
) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    let (reached_tx, reached_rx) = tokio::sync::oneshot::channel();
    let (release_tx, release_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reached_tx = Some(reached_tx);
        for (index, answer) in answers.into_iter().enumerate() {
            let until = std::time::Instant::now() + Duration::from_secs(3);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if std::time::Instant::now() >= until {
                            return;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("mock accept: {e}"),
                }
            };
            stream.set_nonblocking(false).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 4096];
            let req: serde_json::Value = loop {
                let n = stream.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let len: usize = headers
                        .lines()
                        .find_map(|l| {
                            l.to_lowercase()
                                .strip_prefix("content-length: ")
                                .map(str::to_owned)
                        })
                        .unwrap()
                        .parse()
                        .unwrap();
                    if bytes.len() >= end + 4 + len {
                        break serde_json::from_slice(&bytes[end + 4..end + 4 + len]).unwrap();
                    }
                }
            };
            let user: serde_json::Value =
                serde_json::from_str(req["messages"][1]["content"].as_str().unwrap())
                    .unwrap_or_default();
            let job = user["job"].as_str().unwrap_or("legacy");
            let text = if let Some(raw) = answer.as_str() {
                raw.to_owned()
            } else {
                answer.to_string()
            }
            .replace("$JOB", job);
            let frame =
                serde_json::json!({"choices":[{"delta":{"content":text},"finish_reason":null}]});
            let native = req.get("options").is_some();
            tx.send(req).unwrap();
            if pause_request == Some(index + 1) {
                let _ = reached_tx.take().unwrap().send(());
                let _ = release_rx.recv_timeout(Duration::from_secs(10));
            }
            if native {
                let message = serde_json::json!({"message":{"content":text},"done":false});
                let _ = write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: application/x-ndjson\r\nConnection: close\r\n\r\n{message}\n{{\"done\":true}}\n");
            } else {
                let _ = write!(stream,"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\ndata: {frame}\n\ndata: [DONE]\n\n");
            }
        }
    });
    (base, rx, reached_rx, release_tx)
}

pub fn point_packet() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","points":[{"sources":["S000001"]}]})
}
pub fn canonical_point() -> serde_json::Value {
    serde_json::json!({"text":"“Please add a group option.”","evidence":[{"source":"S000001","quote":"Please add a group option."}],"required":[{"source":"S000001","quote":"Please add a group option."}]})
}
pub fn protected_point_packet() -> serde_json::Value {
    point_packet()
}
pub fn protected_evidence() -> serde_json::Value {
    serde_json::json!([{"source":"S000001","quote":"Only when inspection finishes can the item be removed."}])
}
pub fn point_plan() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","status":"supported","blocks":[{"kind":"heading","heading":"overview"},{"kind":"paragraph","claims":["P000001"]}]})
}
pub fn point_report() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","checks":[{"occurrence":"O000001","verdict":"supported","sources":["S000001"]}],"need_sources":[]})
}

pub fn claim() -> serde_json::Value {
    let a = serde_json::json!({"source":"S000001","quote":"Please add a group option."});
    serde_json::json!({"id":"C1","speech_act":{"value":"request","evidence":[a]},"scope":{"value":"planned","evidence":[a]},"certainty":{"value":"asserted","evidence":[a]},"content":{"kind":"point","clause":"Add a group option.","evidence":[a]},"qualifiers":[],"owner":null,"deadline":null,"links":[],"corrections":[]})
}
pub fn packet() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","claims":[claim()]})
}
pub fn report() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","checks":[{"claim":"P1:C1","kind":"point","fields":[
        {"field":"speech_act","verdict":"supported","evidence":[{"source":"S000001","quote":"Please add a group option."}]},
        {"field":"scope","verdict":"supported","evidence":[{"source":"S000001","quote":"Please add a group option."}]},
        {"field":"certainty","verdict":"supported","evidence":[{"source":"S000001","quote":"Please add a group option."}]},
        {"field":"content","verdict":"supported","evidence":[{"source":"S000001","quote":"Please add a group option."}]}
    ]}],"need_sources":[]})
}
pub fn plan() -> serde_json::Value {
    serde_json::json!({"job":"$JOB","status":"supported","blocks":[{"kind":"heading","heading":"overview"},{"kind":"paragraph","claims":["P1:C1"]}]})
}
