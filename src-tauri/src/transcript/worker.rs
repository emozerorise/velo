//! A driver fault or C++ exception must not take the desktop app down.
//! PCM and transcript frames travel over private pipes, never through disk caches.
use super::engine::{self, ProgressFn, TranscriptionDevice};
use super::types::TranscriptSegment;
use crate::errors::{Result, VeloError};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;

const WORKER_ARG: &str = "--velo-transcription-worker";

#[derive(Serialize, Deserialize)]
struct Request {
    model: String,
    language: String,
    prompt: String,
    device: TranscriptionDevice,
    samples: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Frame {
    Progress {
        value: f64,
    },
    Ready {
        language: String,
        segments: Vec<TranscriptSegment>,
    },
    Failed {
        unavailable: bool,
        message: String,
    },
}

fn send(frame: &Frame) {
    let mut stdout = std::io::stdout().lock();
    if serde_json::to_writer(&mut stdout, frame).is_ok() {
        let _ = stdout.write_all(b"\n");
        let _ = stdout.flush();
    }
}

/// Called before Tauri setup in both the app and the diagnostic executable.
pub fn run_if_requested() -> bool {
    if std::env::args().skip(1).collect::<Vec<_>>() != [WORKER_ARG] {
        return false;
    }
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .try_init();
    let result = worker_main();
    match result {
        Ok((language, segments)) => send(&Frame::Ready { language, segments }),
        Err(error) => send(&Frame::Failed {
            unavailable: matches!(error, VeloError::GpuUnavailable),
            message: error.to_string(),
        }),
    }
    true
}

fn worker_main() -> Result<(String, Vec<TranscriptSegment>)> {
    let mut input = BufReader::new(std::io::stdin());
    let mut header = String::new();
    input
        .by_ref()
        .take(64 * 1024)
        .read_line(&mut header)
        .map_err(io_error)?;
    let request: Request =
        serde_json::from_str(&header).map_err(|_| io_error("Invalid worker request"))?;
    let mut samples = Vec::new();
    for _ in 0..request.samples {
        let mut bytes = [0; 4];
        input.read_exact(&mut bytes).map_err(io_error)?;
        samples.push(f32::from_le_bytes(bytes));
    }
    watch_parent(input);
    if request.device == TranscriptionDevice::Gpu && !engine::gpu_available() {
        return Err(VeloError::GpuUnavailable);
    }
    engine::transcribe_with_device(
        Path::new(&request.model),
        &samples,
        &request.language,
        &request.prompt,
        &Arc::new(AtomicBool::new(false)),
        Arc::new(|value| send(&Frame::Progress { value })),
        request.device,
    )
}

fn watch_parent(mut input: impl Read + Send + 'static) {
    // The parent retains its input-pipe writer for the entire job. EOF means
    // the desktop process exited, including while a driver is initializing.
    std::thread::spawn(move || {
        let mut byte = [0];
        loop {
            match input.read(&mut byte) {
                Ok(0) | Err(_) => std::process::exit(0),
                _ => {}
            }
        }
    });
}

struct ReapChild(Child);
impl Drop for ReapChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn io_error(error: impl std::fmt::Display) -> VeloError {
    VeloError::Player(format!("Transcription worker: {error}"))
}

#[allow(clippy::too_many_arguments)]
pub fn transcribe(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
    device: TranscriptionDevice,
) -> Result<(String, Vec<TranscriptSegment>)> {
    let request = Request {
        model: model
            .to_str()
            .ok_or_else(|| io_error("Non-UTF8 model path"))?
            .into(),
        language: language.into(),
        prompt: prompt.into(),
        device,
        samples: samples.len(),
    };
    let mut command = Command::new(std::env::current_exe().map_err(io_error)?);
    command.arg(WORKER_ARG);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    run_child(command, &request, samples, cancel, on_progress)
}

fn run_child(
    mut command: Command,
    request: &Request,
    samples: &[f32],
    cancel: &AtomicBool,
    on_progress: ProgressFn,
) -> Result<(String, Vec<TranscriptSegment>)> {
    if cancel.load(Ordering::Relaxed) {
        return Err(io_error("cancelled"));
    }
    let mut child = ReapChild(
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(io_error)?,
    );
    let stdout = child
        .0
        .stdout
        .take()
        .ok_or_else(|| io_error("Missing output pipe"))?;
    let (sender, receiver) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else {
                break;
            };
            // Some native libraries print diagnostics; only typed frames count.
            if let Ok(frame) = serde_json::from_str::<Frame>(&line) {
                if sender.send(frame).is_err() {
                    break;
                }
            }
        }
    });
    let mut stdin = child
        .0
        .stdin
        .take()
        .ok_or_else(|| io_error("Missing input pipe"))?;
    let request = serde_json::to_vec(request).map_err(io_error)?;
    let samples = samples.to_vec();
    // A blocked child loader/input pipe must remain cancellable too.
    let writer = std::thread::spawn(move || -> std::io::Result<std::process::ChildStdin> {
        stdin.write_all(&request)?;
        stdin.write_all(b"\n")?;
        for chunk in samples.chunks(16_000) {
            let bytes: Vec<u8> = chunk
                .iter()
                .flat_map(|sample| sample.to_le_bytes())
                .collect();
            stdin.write_all(&bytes)?;
        }
        Ok(stdin)
    });
    let mut result = None;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(io_error("cancelled"));
        }
        while let Ok(frame) = receiver.try_recv() {
            match frame {
                Frame::Progress { value } if value.is_finite() => {
                    on_progress(value.clamp(0.0, 1.0))
                }
                Frame::Progress { .. } => {}
                Frame::Ready { language, segments } => result = Some(Ok((language, segments))),
                Frame::Failed {
                    unavailable,
                    message,
                } => {
                    result = Some(Err(if unavailable {
                        VeloError::GpuUnavailable
                    } else {
                        io_error(message)
                    }))
                }
            }
        }
        if let Some(status) = child.0.try_wait().map_err(io_error)? {
            // Drain final frames after EOF; a result is never accepted before
            // a successful exit, including native context teardown.
            let _ = reader.join();
            let _ = writer.join();
            for frame in receiver.try_iter() {
                match frame {
                    Frame::Ready { language, segments } => result = Some(Ok((language, segments))),
                    Frame::Failed {
                        unavailable,
                        message,
                    } => {
                        result = Some(Err(if unavailable {
                            VeloError::GpuUnavailable
                        } else {
                            io_error(message)
                        }))
                    }
                    _ => {}
                }
            }
            if !status.success() {
                return Err(io_error(format!("process exited with {status}")));
            }
            return result.unwrap_or_else(|| Err(io_error("No completed transcript received")));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn fixture(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command.args([
            "--exact",
            "transcript::worker::tests::child_fixture",
            "--nocapture",
        ]);
        command.env("VELO_WORKER_TEST_FIXTURE", mode);
        command
    }
    fn request() -> Request {
        Request {
            model: "unused".into(),
            language: "th".into(),
            prompt: String::new(),
            device: TranscriptionDevice::Gpu,
            samples: 0,
        }
    }
    #[test]
    fn child_fixture() {
        let Ok(mode) = std::env::var("VELO_WORKER_TEST_FIXTURE") else {
            return;
        };
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        println!();
        match mode.as_str() {
            "missing" => send(&Frame::Failed {
                unavailable: true,
                message: "no device".into(),
            }),
            "cancel" => {
                send(&Frame::Progress { value: 0.1 });
                std::thread::sleep(Duration::from_secs(30));
            }
            "disconnect" => {
                watch_parent(std::io::stdin());
                send(&Frame::Progress { value: 0.1 });
                std::thread::sleep(Duration::from_secs(30));
            }
            "crash" => std::process::exit(42),
            "late_crash" => {
                send(&Frame::Ready {
                    language: "th".into(),
                    segments: Vec::new(),
                });
                std::process::exit(42);
            }
            _ => send(&Frame::Ready {
                language: "th".into(),
                segments: vec![TranscriptSegment {
                    start: 0.0,
                    end: 1.0,
                    text: "CPU result".into(),
                }],
            }),
        }
    }
    #[test]
    fn a_crashed_gpu_process_retries_cpu_and_returns_only_the_complete_result() {
        let attempts = RefCell::new(Vec::new());
        let events = RefCell::new(Vec::new());
        let result = engine::run_with_fallback(
            TranscriptionDevice::Auto,
            &AtomicBool::new(false),
            || true,
            |device| {
                attempts.borrow_mut().push(device);
                run_child(
                    fixture(if device == TranscriptionDevice::Gpu {
                        "crash"
                    } else {
                        "cpu"
                    }),
                    &request(),
                    &[],
                    &AtomicBool::new(false),
                    Arc::new(|_| {}),
                )
            },
            |status| events.borrow_mut().push(status),
        );
        assert_eq!(result.unwrap().1[0].text, "CPU result");
        assert_eq!(
            *attempts.borrow(),
            vec![TranscriptionDevice::Gpu, TranscriptionDevice::Cpu]
        );
        assert_eq!(
            events.borrow()[1].fallback,
            Some(engine::FallbackReason::GpuFailed)
        );
    }
    #[test]
    fn a_missing_device_from_the_worker_is_reported_as_cpu_fallback() {
        let events = RefCell::new(Vec::new());
        let result = engine::run_with_fallback(
            TranscriptionDevice::Gpu,
            &AtomicBool::new(false),
            || true,
            |device| {
                run_child(
                    fixture(if device == TranscriptionDevice::Gpu {
                        "missing"
                    } else {
                        "cpu"
                    }),
                    &request(),
                    &[],
                    &AtomicBool::new(false),
                    Arc::new(|_| {}),
                )
            },
            |status| events.borrow_mut().push(status),
        );
        assert_eq!(result.unwrap().1.len(), 1);
        assert_eq!(
            events.borrow()[1].fallback,
            Some(engine::FallbackReason::NoGpu)
        );
    }
    #[test]
    fn a_result_followed_by_native_teardown_failure_is_not_accepted() {
        assert!(run_child(
            fixture("late_crash"),
            &request(),
            &[],
            &AtomicBool::new(false),
            Arc::new(|_| {})
        )
        .is_err());
    }
    #[test]
    fn cancellation_kills_and_reaps_a_running_worker_without_waiting_for_it() {
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        let started = std::time::Instant::now();
        let result = run_child(
            fixture("cancel"),
            &request(),
            &[],
            &cancel,
            Arc::new(move |_| flag.store(true, Ordering::Relaxed)),
        );
        assert!(result.is_err());
        assert!(started.elapsed() < Duration::from_secs(5));
    }
    #[test]
    fn closing_the_parent_pipe_stops_the_child_even_during_initialization() {
        let mut child = ReapChild(
            fixture("disconnect")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let mut input = child.0.stdin.take().unwrap();
        input.write_all(b"{}\n").unwrap();
        let mut output = BufReader::new(child.0.stdout.take().unwrap());
        let mut line = String::new();
        loop {
            line.clear();
            assert!(output.read_line(&mut line).unwrap() > 0);
            if line.contains("progress") {
                break;
            }
        }
        drop(input);
        let started = std::time::Instant::now();
        while child.0.try_wait().unwrap().is_none() {
            assert!(started.elapsed() < Duration::from_secs(5));
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}
