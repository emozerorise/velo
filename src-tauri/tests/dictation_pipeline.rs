//! Synthetic capture adapters only. Never opens a physical microphone.
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use velo_lib::dictation::{
    audio::prepare_audio,
    capture::{record_until_stopped, CaptureBackend, CaptureSession, InputBuffer},
    types::{CapturedAudio, DictationError, Result},
};
use velo_lib::jobs::JobCancel;

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn normalizes_signed_unsigned_float_samples() {
    let signed = InputBuffer::new(16000, 1).unwrap();
    signed.push_input(&[i16::MIN, 0, i16::MAX]);
    let data = signed.finish().unwrap().mono_samples;
    assert_eq!(data[0], -1.0);
    assert_eq!(data[1], 0.0);
    assert!((data[2] - 1.0).abs() < 0.0001);
    let unsigned = InputBuffer::new(16000, 1).unwrap();
    unsigned.push_input(&[0u16, 32768, 65535]);
    let data = unsigned.finish().unwrap().mono_samples;
    assert_eq!(data[0], -1.0);
    assert_eq!(data[1], 0.0);
    assert!((data[2] - 1.0).abs() < 0.0001);
    let floats = InputBuffer::new(16000, 1).unwrap();
    floats.push_input(&[-2.0f64, 0.5, 2.0]);
    assert_eq!(floats.finish().unwrap().mono_samples, [-1.0, 0.5, 1.0]);
}

#[test]
fn stereo_downmix_and_nonfinite_are_bounded() {
    let input = InputBuffer::new(48000, 2).unwrap();
    input.push_mono_frames(&[1.0, -1.0, f32::NAN, f32::INFINITY, -2.0, -2.0], |x| x);
    assert_eq!(input.finish().unwrap().mono_samples, [0.0, 0.0, -1.0]);
    for (rate, channels) in [(7999, 1), (192001, 1), (16000, 0), (16000, 9)] {
        assert!(matches!(
            InputBuffer::new(rate, channels),
            Err(DictationError::UnsupportedFormat)
        ));
    }
}

struct Disconnect(Arc<AtomicUsize>);
struct BrokenSession(Arc<AtomicUsize>);
impl Drop for BrokenSession {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
impl CaptureSession for BrokenSession {
    fn is_complete(&self) -> Result<bool> {
        Err(DictationError::DeviceUnavailable)
    }
    fn stop(self: Box<Self>) -> Result<CapturedAudio> {
        panic!("disconnected capture must be discarded");
    }
}
impl CaptureBackend for Disconnect {
    fn start(&self, _: JobCancel) -> Result<Box<dyn CaptureSession>> {
        Ok(Box::new(BrokenSession(self.0.clone())))
    }
}
#[test]
fn device_disconnect_releases_stream() {
    let drops = Arc::new(AtomicUsize::new(0));
    let result = record_until_stopped(
        &Disconnect(drops.clone()),
        &JobCancel::new(),
        &AtomicBool::new(false),
        |_| {},
    );
    assert!(matches!(result, Err(DictationError::DeviceUnavailable)));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn silence_bypasses_whisper() {
    let input = CapturedAudio {
        sample_rate: 16000,
        mono_samples: vec![0.0; 16000],
    };
    assert!(matches!(prepare_audio(input), Err(DictationError::Silence)));
}

mod lifecycle {
    use super::*;
    use parking_lot::Mutex;
    use std::{future::Future, path::PathBuf, pin::Pin};
    use tokio::sync::{mpsc, oneshot};
    use velo_lib::dictation::{
        permission::{MicPermission, PermissionBackend},
        types::{DictationEvent, DictationPhase},
        DictationInput, DictationState,
    };
    use velo_lib::jobs::{JobGate, JobResource};

    struct Consent(MicPermission);
    impl PermissionBackend for Consent {
        fn request(
            &self,
            _: JobCancel,
        ) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>> {
            let answer = self.0;
            Box::pin(async move { Ok(answer) })
        }
    }
    struct Deferred(Mutex<Option<oneshot::Receiver<MicPermission>>>);
    impl PermissionBackend for Deferred {
        fn request(
            &self,
            _: JobCancel,
        ) -> Pin<Box<dyn Future<Output = Result<MicPermission>> + Send>> {
            let receiver = self.0.lock().take().unwrap();
            Box::pin(async move { receiver.await.map_err(|_| DictationError::Cancelled) })
        }
    }
    struct Capture {
        opens: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
        complete: bool,
        silent: bool,
    }
    struct Session {
        drops: Arc<AtomicUsize>,
        complete: bool,
        silent: bool,
    }
    impl Drop for Session {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }
    impl CaptureSession for Session {
        fn is_complete(&self) -> Result<bool> {
            Ok(self.complete)
        }
        fn stop(self: Box<Self>) -> Result<CapturedAudio> {
            Ok(CapturedAudio {
                sample_rate: 16000,
                mono_samples: vec![if self.silent { 0.0 } else { 0.1 }; 16000],
            })
        }
    }
    impl CaptureBackend for Capture {
        fn start(&self, _: JobCancel) -> Result<Box<dyn CaptureSession>> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Ok(Box::new(Session {
                drops: self.drops.clone(),
                complete: self.complete,
                silent: self.silent,
            }))
        }
    }
    struct Fixture {
        dir: PathBuf,
        state: DictationState,
        gate: JobGate,
        opens: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
        pauses: Arc<AtomicUsize>,
        calls: Arc<AtomicUsize>,
        allowed: Arc<AtomicBool>,
        tx: mpsc::UnboundedSender<DictationEvent>,
        rx: mpsc::UnboundedReceiver<DictationEvent>,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let dir = std::env::temp_dir().join(format!(
                "velo-dictation-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(dir.join("clip.wav"), b"synthetic media identity").unwrap();
            std::fs::write(
                dir.join("model.bin"),
                b"fake engine adapter, not Whisper weights",
            )
            .unwrap();
            let (tx, rx) = mpsc::unbounded_channel();
            Self {
                dir,
                state: DictationState::default(),
                gate: JobGate::default(),
                opens: Arc::default(),
                drops: Arc::default(),
                pauses: Arc::default(),
                calls: Arc::default(),
                allowed: Arc::new(AtomicBool::new(true)),
                tx,
                rx,
            }
        }
        fn input(&self, id: &str) -> DictationInput {
            let calls = self.calls.clone();
            let drops = self.drops.clone();
            DictationInput {
                path: self.dir.join("clip.wav").to_str().unwrap().into(),
                session_id: id.into(),
                model: self.dir.join("model.bin"),
                language: "th".into(),
                vocabulary: "Velo".into(),
                engine: Arc::new(move |_, samples, language, vocabulary, _| {
                    assert_eq!(
                        drops.load(Ordering::SeqCst),
                        1,
                        "capture must be closed before Whisper"
                    );
                    assert_eq!(samples.len(), 16000);
                    assert_eq!(language, "th");
                    assert_eq!(vocabulary, "Velo");
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(" ข้อความสังเคราะห์ ".into())
                }),
            }
        }
        fn start(
            &self,
            input: DictationInput,
            permission: Arc<dyn PermissionBackend>,
            complete: bool,
            silent: bool,
        ) -> Result<()> {
            let pauses = self.pauses.clone();
            let allowed = self.allowed.clone();
            let tx = self.tx.clone();
            self.state.start(
                input,
                permission,
                Arc::new(Capture {
                    opens: self.opens.clone(),
                    drops: self.drops.clone(),
                    complete,
                    silent,
                }),
                Arc::new(move || {
                    pauses.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }),
                Arc::new(move || allowed.load(Ordering::SeqCst)),
                &self.gate,
                Arc::new(move |event| {
                    let _ = tx.send(event);
                }),
            )
        }
        async fn phase(&mut self, expected: DictationPhase) {
            loop {
                if let DictationEvent::State(event) = self.next().await {
                    if event.phase == expected {
                        break;
                    }
                }
            }
        }
        async fn next(&mut self) -> DictationEvent {
            tokio::time::timeout(std::time::Duration::from_secs(2), self.rx.recv())
                .await
                .expect("bounded synthetic worker")
                .unwrap()
        }
        async fn terminal(&mut self) -> DictationEvent {
            loop {
                let event = self.next().await;
                if !matches!(event, DictationEvent::State(_)) {
                    return event;
                }
            }
        }
        async fn released(&self) {
            for _ in 0..100 {
                if self.gate.try_acquire(JobResource::Whisper, "probe").is_ok() {
                    return;
                }
                tokio::task::yield_now().await;
            }
            panic!("worker did not release the speech lease");
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            self.state.cancel_all();
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[tokio::test]
    async fn cancel_while_permission_pending_never_starts_capture() {
        let mut f = Fixture::new();
        let (grant, receiver) = oneshot::channel();
        f.start(
            f.input("pending"),
            Arc::new(Deferred(Mutex::new(Some(receiver)))),
            false,
            false,
        )
        .unwrap();
        f.phase(DictationPhase::RequestingPermission).await;
        f.state.cancel("pending").unwrap();
        let _ = grant.send(MicPermission::Granted);
        assert!(
            matches!(f.terminal().await, DictationEvent::Error(e) if e.code == DictationError::Cancelled && e.cancelled)
        );
        f.released().await;
        assert_eq!(f.opens.load(Ordering::SeqCst), 0);
        assert_eq!(f.pauses.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn permission_denied_preserves_playback() {
        let mut f = Fixture::new();
        f.start(
            f.input("denied"),
            Arc::new(Consent(MicPermission::Denied)),
            false,
            false,
        )
        .unwrap();
        assert!(
            matches!(f.terminal().await, DictationEvent::Error(e) if e.code == DictationError::PermissionDenied)
        );
        assert_eq!(f.pauses.load(Ordering::SeqCst), 0);
        assert_eq!(f.opens.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn missing_model_never_downloads() {
        let f = Fixture::new();
        let mut input = f.input("missing");
        input.model = f.dir.join("missing.bin");
        assert_eq!(
            f.start(
                input,
                Arc::new(Consent(MicPermission::Granted)),
                true,
                false
            ),
            Err(DictationError::MissingModel)
        );
        assert!(!f.dir.join("missing.bin").exists());
        assert_eq!(f.opens.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn recording_pauses_once_without_resume() {
        let mut f = Fixture::new();
        f.start(
            f.input("record"),
            Arc::new(Consent(MicPermission::Granted)),
            false,
            false,
        )
        .unwrap();
        f.phase(DictationPhase::Recording).await;
        assert_eq!(f.pauses.load(Ordering::SeqCst), 1);
        f.state.stop("record").unwrap();
        f.state.stop("record").unwrap();
        assert!(
            matches!(f.terminal().await, DictationEvent::Ready(e) if e.text == "ข้อความสังเคราะห์" && e.session_id == "record")
        );
        f.released().await;
        assert_eq!(f.drops.load(Ordering::SeqCst), 1);
        assert_eq!(f.pauses.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn stop_timeout_cancel_has_one_terminal_event() {
        for cancel in [false, true] {
            let mut f = Fixture::new();
            f.start(
                f.input("race"),
                Arc::new(Consent(MicPermission::Granted)),
                !cancel,
                false,
            )
            .unwrap();
            if cancel {
                f.phase(DictationPhase::Recording).await;
                f.state.stop("race").unwrap();
                f.state.cancel("race").unwrap();
            }
            let _ = f.terminal().await;
            f.released().await;
            f.state.stop("race").unwrap();
            f.state.cancel("race").unwrap();
            assert!(f.rx.try_recv().is_err());
        }
    }
    #[tokio::test]
    async fn engine_busy_rejects_without_device_open() {
        let f = Fixture::new();
        let _file = f
            .gate
            .try_acquire(JobResource::Whisper, "file-transcript")
            .unwrap();
        assert_eq!(
            f.start(
                f.input("busy"),
                Arc::new(Consent(MicPermission::Granted)),
                true,
                false
            ),
            Err(DictationError::EngineBusy)
        );
        assert_eq!(f.opens.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn close_during_whisper_discards_result_and_releases_lease() {
        let mut f = Fixture::new();
        let (entered, started) = oneshot::channel();
        let entered = Mutex::new(Some(entered));
        let release = Arc::new(AtomicBool::new(false));
        let wait = release.clone();
        let mut input = f.input("close");
        input.engine = Arc::new(move |_, _, _, _, _| {
            let _ = entered.lock().take().unwrap().send(());
            while !wait.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Ok("late private text".into())
        });
        f.start(
            input,
            Arc::new(Consent(MicPermission::Granted)),
            true,
            false,
        )
        .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), started)
            .await
            .unwrap()
            .unwrap();
        f.state.cancel_all();
        assert!(f
            .gate
            .try_acquire(JobResource::Whisper, "premature")
            .is_err());
        release.store(true, Ordering::Release);
        assert!(
            matches!(f.terminal().await, DictationEvent::Error(e) if e.code == DictationError::Cancelled)
        );
        f.released().await;
    }
    #[tokio::test]
    async fn dictation_does_not_write_transcript_cache() {
        let mut f = Fixture::new();
        std::fs::write(
            f.dir.join("cached-transcript.json"),
            b"original cached source",
        )
        .unwrap();
        let before = std::fs::read(f.dir.join("cached-transcript.json")).unwrap();
        f.start(
            f.input("memory"),
            Arc::new(Consent(MicPermission::Granted)),
            true,
            false,
        )
        .unwrap();
        assert!(matches!(f.terminal().await, DictationEvent::Ready(_)));
        f.released().await;
        assert_eq!(
            std::fs::read(f.dir.join("cached-transcript.json")).unwrap(),
            before
        );
        assert_eq!(std::fs::read_dir(&f.dir).unwrap().count(), 3);
        assert_eq!(f.calls.load(Ordering::SeqCst), 1);
    }
    #[tokio::test]
    async fn permission_grant_in_unfocused_window_never_captures() {
        let mut f = Fixture::new();
        f.allowed.store(false, Ordering::Release);
        f.start(
            f.input("unfocused"),
            Arc::new(Consent(MicPermission::Granted)),
            true,
            false,
        )
        .unwrap();
        assert!(matches!(f.terminal().await, DictationEvent::Error(e) if e.cancelled));
        assert_eq!(f.opens.load(Ordering::SeqCst), 0);
        assert_eq!(f.pauses.load(Ordering::SeqCst), 0);
    }
    #[tokio::test]
    async fn silent_capture_never_invokes_engine() {
        let mut f = Fixture::new();
        f.start(
            f.input("silent"),
            Arc::new(Consent(MicPermission::Granted)),
            true,
            true,
        )
        .unwrap();
        assert!(
            matches!(f.terminal().await, DictationEvent::Error(e) if e.code == DictationError::Silence)
        );
        assert_eq!(f.calls.load(Ordering::SeqCst), 0);
    }
}
