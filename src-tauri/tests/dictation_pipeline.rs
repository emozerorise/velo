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
