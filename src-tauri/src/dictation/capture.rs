use super::audio::finite_sample;
use super::types::{CapturedAudio, DictationError, Result, MAX_SECONDS};
use crate::jobs::JobCancel;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::{Duration, Instant};

pub trait CaptureBackend: Send + Sync {
    fn start(&self, cancel: JobCancel) -> Result<Box<dyn CaptureSession>>;
}

/// Deliberately not Send: the capture worker owns the native stream and drops it.
pub trait CaptureSession {
    fn stop(self: Box<Self>) -> Result<CapturedAudio>;
    fn is_complete(&self) -> Result<bool> {
        Ok(false)
    }
}

pub struct InputBuffer {
    sample_rate: u32,
    channels: usize,
    limit: usize,
    samples: Mutex<Vec<f32>>,
    error: AtomicU8,
    complete: AtomicBool,
    closed: AtomicBool,
}

impl InputBuffer {
    pub fn new(sample_rate: u32, channels: u16) -> Result<Self> {
        if !(8000..=192000).contains(&sample_rate) || !(1..=8).contains(&channels) {
            return Err(DictationError::UnsupportedFormat);
        }
        let limit = sample_rate as usize * MAX_SECONDS;
        let mut samples = Vec::new();
        samples
            .try_reserve_exact(limit)
            .map_err(|_| DictationError::CaptureFailed)?;
        Ok(Self {
            sample_rate,
            channels: usize::from(channels),
            limit,
            samples: Mutex::new(samples),
            error: AtomicU8::new(0),
            complete: AtomicBool::new(false),
            closed: AtomicBool::new(false),
        })
    }

    pub fn push_mono_frames<T: Copy>(&self, input: &[T], convert: impl Fn(T) -> f32) {
        if self.closed.load(Ordering::Acquire)
            || self.complete.load(Ordering::Acquire)
            || self.error.load(Ordering::Acquire) != 0
        {
            return;
        }
        if !input.len().is_multiple_of(self.channels) {
            self.fail(false);
            return;
        }
        let Some(mut samples) = self.samples.try_lock() else {
            self.fail(false);
            return;
        };
        for frame in input.chunks_exact(self.channels) {
            if samples.len() == self.limit {
                self.complete.store(true, Ordering::Release);
                break;
            }
            let mono = frame
                .iter()
                .map(|&value| finite_sample(convert(value)))
                .sum::<f32>()
                / self.channels as f32;
            samples.push(mono);
        }
        if samples.len() == self.limit {
            self.complete.store(true, Ordering::Release);
        }
    }

    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn push_input<T: cpal::Sample>(&self, input: &[T])
    where
        f32: cpal::FromSample<T>,
    {
        self.push_mono_frames(input, |value| value.to_sample::<f32>());
    }

    fn fail(&self, disconnected: bool) {
        let _ = self.error.compare_exchange(
            0,
            if disconnected { 2 } else { 1 },
            Ordering::AcqRel,
            Ordering::Acquire,
        );
    }

    pub fn is_complete(&self) -> Result<bool> {
        match self.error.load(Ordering::Acquire) {
            0 => Ok(self.complete.load(Ordering::Acquire)),
            2 => Err(DictationError::DeviceUnavailable),
            _ => Err(DictationError::CaptureFailed),
        }
    }

    pub fn finish(&self) -> Result<CapturedAudio> {
        self.closed.store(true, Ordering::Release);
        let mut samples = self.samples.lock();
        self.is_complete()?;
        Ok(CapturedAudio {
            sample_rate: self.sample_rate,
            mono_samples: std::mem::take(&mut *samples),
        })
    }
}

pub fn record_until_stopped(
    backend: &dyn CaptureBackend,
    cancel: &JobCancel,
    stop: &AtomicBool,
    progress: impl FnMut(u64),
) -> Result<CapturedAudio> {
    record_with_limit(
        backend,
        cancel,
        stop,
        progress,
        Duration::from_secs(MAX_SECONDS as u64),
    )
}

fn record_with_limit(
    backend: &dyn CaptureBackend,
    cancel: &JobCancel,
    stop: &AtomicBool,
    mut progress: impl FnMut(u64),
    limit: Duration,
) -> Result<CapturedAudio> {
    if cancel.flag().load(Ordering::Acquire) {
        return Err(DictationError::Cancelled);
    }
    let started = Instant::now();
    let session = backend.start(cancel.clone())?;
    let mut last_report = 0;
    progress(0);
    loop {
        if cancel.flag().load(Ordering::Acquire) {
            return Err(DictationError::Cancelled);
        }
        let elapsed = started.elapsed();
        if session.is_complete()?
            || stop.load(Ordering::Acquire)
            || elapsed >= limit.min(Duration::from_secs(MAX_SECONDS as u64))
        {
            return session.stop();
        }
        let seconds = elapsed.as_secs();
        if seconds != last_report {
            last_report = seconds;
            progress(elapsed.as_millis() as u64);
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[derive(Default)]
pub struct NativeCapture;

#[cfg(any(target_os = "macos", target_os = "windows"))]
fn native_error(error: impl std::fmt::Display, fallback: DictationError) -> DictationError {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("80070005")
        || message.contains("access is denied")
        || message.contains("permission denied")
        || message.contains("operation not permitted")
    {
        DictationError::PermissionDenied
    } else {
        fallback
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod native {
    use super::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::Arc;
    struct NativeSession {
        stream: Option<cpal::Stream>,
        buffer: Arc<InputBuffer>,
    }
    impl CaptureSession for NativeSession {
        fn is_complete(&self) -> Result<bool> {
            self.buffer.is_complete()
        }
        fn stop(mut self: Box<Self>) -> Result<CapturedAudio> {
            drop(self.stream.take());
            self.buffer.finish()
        }
    }
    fn stream<T: cpal::SizedSample>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<InputBuffer>,
        cancel: JobCancel,
    ) -> Result<cpal::Stream>
    where
        f32: cpal::FromSample<T>,
    {
        let errors = buffer.clone();
        device
            .build_input_stream(
                config,
                move |input: &[T], _| {
                    if !cancel.flag().load(Ordering::Acquire) {
                        buffer.push_input(input);
                    }
                },
                move |_| errors.fail(true),
                None,
            )
            .map_err(|error| match error {
                cpal::BuildStreamError::StreamConfigNotSupported => {
                    DictationError::UnsupportedFormat
                }
                _ => native_error(error, DictationError::CaptureFailed),
            })
    }
    impl CaptureBackend for NativeCapture {
        fn start(&self, cancel: JobCancel) -> Result<Box<dyn CaptureSession>> {
            if cancel.flag().load(Ordering::Acquire) {
                return Err(DictationError::Cancelled);
            }
            // No host/device enumeration occurs until an approved start reaches here.
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or(DictationError::DeviceUnavailable)?;
            let supported = device
                .default_input_config()
                .map_err(|error| native_error(error, DictationError::DeviceUnavailable))?;
            let format = supported.sample_format();
            let config: cpal::StreamConfig = supported.into();
            let buffer = Arc::new(InputBuffer::new(config.sample_rate.0, config.channels)?);
            if cancel.flag().load(Ordering::Acquire) {
                return Err(DictationError::Cancelled);
            }
            macro_rules! build {
                ($sample:ty) => {
                    stream::<$sample>(&device, &config, buffer.clone(), cancel.clone())?
                };
            }
            let stream = match format {
                cpal::SampleFormat::I8 => build!(i8),
                cpal::SampleFormat::I16 => build!(i16),
                cpal::SampleFormat::I32 => build!(i32),
                cpal::SampleFormat::I64 => build!(i64),
                cpal::SampleFormat::U8 => build!(u8),
                cpal::SampleFormat::U16 => build!(u16),
                cpal::SampleFormat::U32 => build!(u32),
                cpal::SampleFormat::U64 => build!(u64),
                cpal::SampleFormat::F32 => build!(f32),
                cpal::SampleFormat::F64 => build!(f64),
                _ => return Err(DictationError::UnsupportedFormat),
            };
            stream
                .play()
                .map_err(|error| native_error(error, DictationError::CaptureFailed))?;
            Ok(Box::new(NativeSession {
                stream: Some(stream),
                buffer,
            }))
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl CaptureBackend for NativeCapture {
    fn start(&self, _: JobCancel) -> Result<Box<dyn CaptureSession>> {
        Err(DictationError::DeviceUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    #[test]
    fn native_access_denial_is_a_private_stable_code() {
        assert_eq!(
            native_error(
                "WASAPI Activate failed: HRESULT 0x80070005",
                DictationError::CaptureFailed
            ),
            DictationError::PermissionDenied
        );
        assert_eq!(
            native_error(
                "synthetic device path: disconnected",
                DictationError::DeviceUnavailable
            ),
            DictationError::DeviceUnavailable
        );
    }
    #[test]
    fn wall_clock_deadline_stops_without_waiting_for_more_samples() {
        use std::sync::{atomic::AtomicUsize, Arc};
        struct Backend(Arc<AtomicUsize>);
        struct Session(Arc<AtomicUsize>);
        impl Drop for Session {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        impl CaptureSession for Session {
            fn stop(self: Box<Self>) -> Result<CapturedAudio> {
                Ok(CapturedAudio {
                    sample_rate: 16000,
                    mono_samples: vec![0.1],
                })
            }
        }
        impl CaptureBackend for Backend {
            fn start(&self, _: JobCancel) -> Result<Box<dyn CaptureSession>> {
                Ok(Box::new(Session(self.0.clone())))
            }
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let observed = drops.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = record_with_limit(
                &Backend(observed),
                &JobCancel::new(),
                &AtomicBool::new(false),
                |_| {},
                Duration::from_millis(1),
            );
            let _ = tx.send(result);
        });
        let audio = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("wall-clock limit must close a stalled device")
            .unwrap();
        assert_eq!(audio.mono_samples.len(), 1);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    #[test]
    fn capture_limit_never_reallocates() {
        let buffer = InputBuffer::new(8000, 1).unwrap();
        let capacity = buffer.samples.lock().capacity();
        let ptr = buffer.samples.lock().as_ptr();
        for _ in 0..470 {
            buffer.push_mono_frames(&[0.1; 1024], |x| x);
        }
        assert_eq!(buffer.samples.lock().capacity(), capacity);
        assert_eq!(buffer.samples.lock().as_ptr(), ptr);
        assert_eq!(buffer.samples.lock().len(), 480000);
        assert!(buffer.is_complete().unwrap());
        assert_eq!(buffer.finish().unwrap().mono_samples.len(), 480000);
        let max = InputBuffer::new(192000, 8).unwrap();
        assert_eq!(
            max.samples.lock().capacity() * std::mem::size_of::<f32>(),
            46080000
        );
    }
    #[test]
    fn callback_contention_reports_error() {
        let buffer = InputBuffer::new(16000, 1).unwrap();
        let guard = buffer.samples.lock();
        buffer.push_mono_frames(&[0.1], |x| x);
        drop(guard);
        assert!(matches!(
            buffer.is_complete(),
            Err(DictationError::CaptureFailed)
        ));
        assert!(buffer.finish().is_err());
    }
}
