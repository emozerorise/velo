//! Loopback-only model double; no credentials, cloud calls or quality oracle.
use serde_json::{json, Value};
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Reply {
    pub text: String,
    pub finish: &'static str,
    pub status: u16,
}

impl Reply {
    pub fn complete(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            finish: "stop",
            status: 200,
        }
    }
    pub fn limited() -> Self {
        Self {
            text: "UNFINISHED RESPONSE".into(),
            finish: "length",
            status: 200,
        }
    }
}

pub struct Model {
    pub base: String,
    pub requests: Arc<Mutex<Vec<Value>>>,
    stop: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl Model {
    pub fn start(mut reply: impl FnMut(&Value, usize) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let stop = Arc::new(AtomicBool::new(false));
        let requests = Arc::new(Mutex::new(Vec::new()));
        let (stopped, received) = (stop.clone(), requests.clone());
        let worker = thread::spawn(move || {
            while !stopped.load(Ordering::Acquire) {
                let (mut stream, _) = match listener.accept() {
                    Ok(connection) => connection,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(error) => panic!("mock accept: {error}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut bytes = Vec::new();
                let mut buf = [0; 4096];
                let request: Value = loop {
                    let count = stream.read(&mut buf).unwrap();
                    assert_ne!(count, 0, "incomplete mock request");
                    bytes.extend_from_slice(&buf[..count]);
                    if let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&bytes[..end]);
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length: ")
                                    .map(str::to_owned)
                            })
                            .unwrap()
                            .parse()
                            .unwrap();
                        if bytes.len() >= end + 4 + length {
                            break serde_json::from_slice(&bytes[end + 4..end + 4 + length])
                                .unwrap();
                        }
                    }
                };
                let index = {
                    let mut all = received.lock().unwrap();
                    let index = all.len();
                    all.push(request.clone());
                    index
                };
                let response = reply(&request, index);
                let delta =
                    json!({"choices":[{"delta":{"content":response.text},"finish_reason":null}]});
                let done = json!({"choices":[{"delta":{},"finish_reason":response.finish}]});
                let native = request.get("options").is_some();
                let body = if native {
                    format!(
                        "{}\n{}\n",
                        json!({"message":{"content":response.text},"done":false}),
                        json!({"done":true,"done_reason":response.finish})
                    )
                } else {
                    format!("data: {delta}\n\ndata: {done}\n\ndata: [DONE]\n\n")
                };
                let _ = write!(stream, "HTTP/1.1 {} Mock\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", response.status, body.len(), body);
            }
        });
        Self {
            base,
            requests,
            stop,
            worker: Some(worker),
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let result = worker.join();
            if !thread::panicking() {
                result.unwrap();
            }
        }
    }
}
