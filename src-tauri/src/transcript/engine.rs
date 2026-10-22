//! In-process transcription with whisper.cpp.
//!
//! whisper is linked into the binary rather than shelled out to, so a release
//! only needs the model file, which `super::model` fetches on first use.

use crate::errors::{Result, VeloError};
use crate::transcript::types::TranscriptSegment;
use std::cell::Cell;
use std::ffi::{c_void, CStr};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Once;
use whisper_rs::whisper_rs_sys as sys;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const SAMPLE_RATE: usize = 16_000;

/// whisper only conditions on its initial prompt for the first 30-second
/// window, but a meeting's vocabulary matters just as much an hour in.
/// Feeding the audio in chunks and re-supplying the prompt for each one
/// reproduces the CLI's `--carry-initial-prompt`, which measurements on a real
/// Thai meeting showed is where nearly all of the prompt's benefit comes from:
/// without it, spoken English terms come back as Thai phonetic spellings
/// almost as often as with no prompt at all.
const CHUNK_SECONDS: usize = 300;

/// Progress reporting has to be owned rather than borrowed: whisper-rs
/// requires `'static` callbacks, since they are handed to C.
pub type ProgressFn = Arc<dyn Fn(f64) + Send + Sync>;

static LOGGING_HOOK: Once = Once::new();
thread_local! {
    // whisper has no public getter for the backend a context actually uses.
    // Its synchronous initialization diagnostic distinguishes a GPU context
    // from the library's own successful but otherwise invisible CPU fallback.
    static GPU_INITIALIZED: Cell<bool> = const { Cell::new(false) };
}

fn observe_backend(message: &str) {
    if message.contains("whisper_backend_init_gpu: failed to initialize ") {
        let _ = GPU_INITIALIZED.try_with(|flag| flag.set(false));
    } else if message.contains("whisper_backend_init_gpu: using ") && message.contains(" backend") {
        let _ = GPU_INITIALIZED.try_with(|flag| flag.set(true));
    }
}

// The callback is installed once for the process, never with a job-owned
// pointer. Even a logging subscriber panic must not unwind across the C ABI.
unsafe extern "C" fn whisper_log(
    level: sys::ggml_log_level,
    text: *const std::ffi::c_char,
    _: *mut c_void,
) {
    if text.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(|| {
        // SAFETY: whisper supplies a NUL-terminated string for this callback.
        let message = unsafe { CStr::from_ptr(text) }.to_string_lossy();
        observe_backend(&message);
        match whisper_rs::GGMLLogLevel::from(level) {
            whisper_rs::GGMLLogLevel::Error => {
                tracing::error!(target: "velo::whisper", "{}", message.trim())
            }
            whisper_rs::GGMLLogLevel::Warn => {
                tracing::warn!(target: "velo::whisper", "{}", message.trim())
            }
            whisper_rs::GGMLLogLevel::Info => {
                tracing::info!(target: "velo::whisper", "{}", message.trim())
            }
            _ => tracing::debug!(target: "velo::whisper", "{}", message.trim()),
        }
    });
}

fn init_logging() {
    LOGGING_HOOK.call_once(|| {
        whisper_rs::install_logging_hooks();
        // SAFETY: static callback, no borrowed user data, no unwinding.
        unsafe {
            whisper_rs::set_log_callback(Some(whisper_log), std::ptr::null_mut());
        }
    });
}

/// whisper polls this between compute steps; returning true aborts the run.
///
/// # Safety
/// `user_data` must be a pointer to an `AtomicBool` that outlives the
/// `whisper_full` call it was handed to.
unsafe extern "C" fn abort_if_cancelled(user_data: *mut c_void) -> bool {
    if user_data.is_null() {
        return false;
    }
    unsafe { (*(user_data as *const AtomicBool)).load(Ordering::Relaxed) }
}

/// Transcribe 16 kHz mono samples, reporting 0.0..1.0 overall progress.
///
/// Returns the language whisper settled on and the segments, timestamped
/// against the start of the media rather than the start of a chunk.
pub fn transcribe(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
) -> Result<(String, Vec<TranscriptSegment>)> {
    transcribe_with_device(
        model,
        samples,
        language,
        prompt,
        cancel,
        on_progress,
        TranscriptionDevice::Auto,
    )
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranscriptionDevice {
    #[default]
    Auto,
    Gpu,
    Cpu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackReason {
    NoGpu,
    GpuFailed,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ExecutionStatus {
    pub device: TranscriptionDevice,
    pub fallback: Option<FallbackReason>,
}

pub type DeviceFn = Arc<dyn Fn(ExecutionStatus) + Send + Sync>;

/// GPU preference is best-effort in both Auto and GPU mode. CPU mode must
/// bypass even device discovery so a broken GPU driver cannot affect it.
#[allow(clippy::too_many_arguments)]
pub fn transcribe_with_fallback(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
    device: TranscriptionDevice,
    on_device: DeviceFn,
) -> Result<(String, Vec<TranscriptSegment>)> {
    if cancel.load(Ordering::Relaxed) {
        return Err(VeloError::Player("Transcription cancelled".into()));
    }
    if samples.is_empty() {
        return Err(VeloError::Player("There is no audio to transcribe".into()));
    }
    run_with_fallback(
        device,
        cancel,
        || true,
        |selected| {
            super::worker::transcribe(
                model,
                samples,
                language,
                prompt,
                cancel,
                on_progress.clone(),
                selected,
            )
        },
        |status| {
            on_progress(0.0);
            on_device(status);
        },
    )
}

pub fn gpu_available() -> bool {
    init_logging();
    // The registry catches Vulkan loader/ICD initialization failures. The
    // higher-level Vulkan list_devices also creates buffers and can throw
    // across the FFI boundary on unsupported drivers, so do not use it here.
    // SAFETY: registry handles are process-owned; indexes are bounded by count.
    unsafe {
        sys::ggml_backend_load_all();
        (0..sys::ggml_backend_dev_count()).any(|index| {
            let device = sys::ggml_backend_dev_get(index);
            matches!(
                sys::ggml_backend_dev_type(device),
                sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_GPU
                    | sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_IGPU
            )
        })
    }
}

pub(super) fn run_with_fallback<T>(
    device: TranscriptionDevice,
    cancel: &AtomicBool,
    available: impl FnOnce() -> bool,
    mut attempt: impl FnMut(TranscriptionDevice) -> Result<T>,
    notify: impl Fn(ExecutionStatus),
) -> Result<T> {
    if device == TranscriptionDevice::Cpu {
        notify(ExecutionStatus {
            device: TranscriptionDevice::Cpu,
            fallback: None,
        });
        return attempt(TranscriptionDevice::Cpu);
    }
    if !available() {
        notify(ExecutionStatus {
            device: TranscriptionDevice::Cpu,
            fallback: Some(FallbackReason::NoGpu),
        });
        return attempt(TranscriptionDevice::Cpu);
    }
    notify(ExecutionStatus {
        device: TranscriptionDevice::Gpu,
        fallback: None,
    });
    match attempt(TranscriptionDevice::Gpu) {
        Ok(result) => Ok(result),
        Err(error) if cancel.load(Ordering::Relaxed) => Err(error),
        Err(error) => {
            tracing::warn!(%error, "GPU transcription failed; retrying once on CPU");
            notify(ExecutionStatus {
                device: TranscriptionDevice::Cpu,
                fallback: Some(if matches!(error, VeloError::GpuUnavailable) {
                    FallbackReason::NoGpu
                } else {
                    FallbackReason::GpuFailed
                }),
            });
            attempt(TranscriptionDevice::Cpu)
        }
    }
}

/// Uses the same decoding settings on either backend for local diagnostics.
/// This lower-level entry deliberately does not retry, so benchmarks cannot
/// silently label CPU execution as a successful GPU measurement.
pub fn transcribe_with_device(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
    device: TranscriptionDevice,
) -> Result<(String, Vec<TranscriptSegment>)> {
    transcribe_chunked_with_device(
        model,
        samples,
        language,
        prompt,
        cancel,
        on_progress,
        CHUNK_SECONDS,
        device,
    )
}
/// The body of [`transcribe`], with the chunk length exposed: it is what
/// carries the prompt and offsets timestamps, and covering it at the real
/// five-minute default would need a fixture longer than any test wants to be.
#[allow(clippy::too_many_arguments)]
pub fn transcribe_chunked(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
    chunk_seconds: usize,
) -> Result<(String, Vec<TranscriptSegment>)> {
    transcribe_chunked_with_device(
        model,
        samples,
        language,
        prompt,
        cancel,
        on_progress,
        chunk_seconds,
        TranscriptionDevice::Auto,
    )
}

#[allow(clippy::too_many_arguments)]
fn transcribe_chunked_with_device(
    model: &Path,
    samples: &[f32],
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: ProgressFn,
    chunk_seconds: usize,
    device: TranscriptionDevice,
) -> Result<(String, Vec<TranscriptSegment>)> {
    if samples.is_empty() {
        return Err(VeloError::Player("There is no audio to transcribe".into()));
    }

    // Without this, ggml and whisper write their own diagnostics straight to
    // stderr, outside the app's tracing setup.
    init_logging();
    GPU_INITIALIZED.with(|flag| flag.set(false));

    let mut context_params = WhisperContextParameters::default();
    if matches!(device, TranscriptionDevice::Cpu) {
        context_params.use_gpu(false);
    }
    let ctx = WhisperContext::new_with_params(
        model
            .to_str()
            .ok_or_else(|| VeloError::InvalidParameter("Non-UTF8 model path".into()))?,
        context_params,
    )
    .map_err(|e| VeloError::Player(format!("Could not load the whisper model: {}", e)))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| VeloError::Player(format!("Could not start whisper: {}", e)))?;
    if device == TranscriptionDevice::Gpu && !GPU_INITIALIZED.with(Cell::get) {
        return Err(VeloError::Player(
            "Whisper could not initialize its GPU backend".into(),
        ));
    }

    let threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1))
        .unwrap_or(4);

    let chunk_len = chunk_seconds.max(1) * SAMPLE_RATE;
    let total_samples = samples.len() as f64;
    let prompt = prompt.trim();

    let mut segments = Vec::new();
    let mut detected = String::new();

    for (index, chunk) in samples.chunks(chunk_len).enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(VeloError::Player("Transcription cancelled".into()));
        }

        let chunk_start = (index * chunk_len) as f64 / SAMPLE_RATE as f64;
        let done_before = (index * chunk_len) as f64 / total_samples;
        let chunk_share = chunk.len() as f64 / total_samples;

        let mut params = FullParams::new(SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_n_threads(threads as i32);
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        // "auto" is a language whisper understands, and detection then happens
        // as part of transcribing. `set_detect_language` is a different thing
        // entirely: it makes whisper report the language and return without
        // producing a single segment.
        params.set_language(Some(language));

        if !prompt.is_empty() {
            params.set_initial_prompt(prompt);
        }

        let progress = on_progress.clone();
        params.set_progress_callback_safe(move |percent: i32| {
            let fraction = done_before + chunk_share * (percent as f64 / 100.0);
            progress(fraction.clamp(0.0, 1.0));
        });

        // whisper checks this between compute steps, so a cancelled job stops
        // in seconds instead of running to the end of an eight-hour file.
        //
        // The raw setters rather than `set_abort_callback_safe`: that helper
        // stores a `Box<Box<dyn FnMut>>` but instantiates its trampoline for
        // the *closure* type, so the callback reinterprets a fat pointer as
        // the closure and whisper aborts immediately with "failed to encode".
        // (Its progress equivalent gets this right, which is why that one is
        // used above.)
        unsafe {
            params.set_abort_callback(Some(abort_if_cancelled));
            params.set_abort_callback_user_data(Arc::as_ptr(cancel) as *mut c_void);
        }

        state
            .full(params, chunk)
            .map_err(|e| VeloError::Player(format!("Transcription failed: {}", e)))?;

        if detected.is_empty() {
            if let Some(lang) = whisper_rs::get_lang_str(state.full_lang_id_from_state()) {
                detected = lang.to_string();
            }
        }

        // whisper pads its input out to a 30-second window boundary and will
        // happily produce segments inside that padding, timed past the end of
        // the audio it was given. Left alone those become transcript lines
        // that seek past the end of the video.
        let chunk_end = chunk_start + chunk.len() as f64 / SAMPLE_RATE as f64;

        for segment in state.as_iter() {
            let text = segment
                .to_str_lossy()
                .map(|t| t.trim().to_string())
                .unwrap_or_default();
            if text.is_empty() {
                continue;
            }

            let start = chunk_start + segment.start_timestamp() as f64 / 100.0;
            if start >= chunk_end {
                continue;
            }

            segments.push(TranscriptSegment {
                start,
                end: (chunk_start + segment.end_timestamp() as f64 / 100.0).min(chunk_end),
                text,
            });
        }
    }

    on_progress(1.0);
    Ok((detected, segments))
}

#[cfg(all(test, target_os = "windows"))]
mod gpu_build_tests {
    use super::*;

    #[test]
    fn windows_transcription_build_enables_gpu() {
        assert!(
            WhisperContextParameters::default().use_gpu,
            "Windows transcription was built without a GPU backend"
        );
    }
}

#[cfg(test)]
mod fallback_tests {
    #[test]
    fn failed_native_gpu_initialization_clears_the_optimistic_log_marker() {
        super::GPU_INITIALIZED.with(|flag| flag.set(false));
        super::observe_backend("whisper_backend_init_gpu: using Vulkan0 backend");
        assert!(super::GPU_INITIALIZED.with(std::cell::Cell::get));
        super::observe_backend("whisper_backend_init_gpu: failed to initialize Vulkan0 backend");
        assert!(!super::GPU_INITIALIZED.with(std::cell::Cell::get));
    }

    use super::*;
    use std::cell::RefCell;

    #[test]
    fn cpu_mode_never_probes_or_attempts_gpu() {
        let attempts = RefCell::new(Vec::new());
        let events = RefCell::new(Vec::new());
        let result = run_with_fallback(
            TranscriptionDevice::Cpu,
            &AtomicBool::new(false),
            || panic!("CPU mode must not load GPU drivers"),
            |device| {
                attempts.borrow_mut().push(device);
                Ok("cpu result")
            },
            |status| events.borrow_mut().push(status),
        );
        assert_eq!(result.unwrap(), "cpu result");
        assert_eq!(*attempts.borrow(), vec![TranscriptionDevice::Cpu]);
        assert_eq!(events.borrow()[0].fallback, None);
    }

    #[test]
    fn missing_gpu_uses_cpu_once_and_explains_the_fallback_in_both_modes() {
        for mode in [TranscriptionDevice::Auto, TranscriptionDevice::Gpu] {
            let attempts = RefCell::new(Vec::new());
            let events = RefCell::new(Vec::new());
            let result = run_with_fallback(
                mode,
                &AtomicBool::new(false),
                || false,
                |device| {
                    attempts.borrow_mut().push(device);
                    Ok(42)
                },
                |status| events.borrow_mut().push(status),
            );
            assert_eq!(result.unwrap(), 42);
            assert_eq!(*attempts.borrow(), vec![TranscriptionDevice::Cpu]);
            assert_eq!(events.borrow()[0].fallback, Some(FallbackReason::NoGpu));
        }
    }

    #[test]
    fn gpu_failure_restarts_on_cpu_once_and_only_returns_the_cpu_result() {
        for mode in [TranscriptionDevice::Auto, TranscriptionDevice::Gpu] {
            let attempts = RefCell::new(Vec::new());
            let events = RefCell::new(Vec::new());
            let result = run_with_fallback(
                mode,
                &AtomicBool::new(false),
                || true,
                |device| {
                    attempts.borrow_mut().push(device);
                    match device {
                        TranscriptionDevice::Gpu => {
                            Err(VeloError::Player("GPU allocation failed".into()))
                        }
                        _ => Ok("complete CPU transcript"),
                    }
                },
                |status| events.borrow_mut().push(status),
            );
            assert_eq!(result.unwrap(), "complete CPU transcript");
            assert_eq!(
                *attempts.borrow(),
                vec![TranscriptionDevice::Gpu, TranscriptionDevice::Cpu]
            );
            assert_eq!(events.borrow()[1].device, TranscriptionDevice::Cpu);
            assert_eq!(events.borrow()[1].fallback, Some(FallbackReason::GpuFailed));
        }
    }

    #[test]
    fn successful_gpu_is_not_followed_by_cpu() {
        let attempts = RefCell::new(Vec::new());
        let result = run_with_fallback(
            TranscriptionDevice::Auto,
            &AtomicBool::new(false),
            || true,
            |device| {
                attempts.borrow_mut().push(device);
                Ok(7)
            },
            |_| {},
        );
        assert_eq!(result.unwrap(), 7);
        assert_eq!(*attempts.borrow(), vec![TranscriptionDevice::Gpu]);
    }

    #[test]
    fn cancellation_during_gpu_failure_never_starts_cpu() {
        let cancel = AtomicBool::new(false);
        let attempts = RefCell::new(Vec::new());
        let result: Result<()> = run_with_fallback(
            TranscriptionDevice::Auto,
            &cancel,
            || true,
            |device| {
                attempts.borrow_mut().push(device);
                cancel.store(true, Ordering::Relaxed);
                Err(VeloError::Player("cancelled".into()))
            },
            |_| {},
        );
        assert!(result.is_err());
        assert_eq!(*attempts.borrow(), vec![TranscriptionDevice::Gpu]);
    }

    #[test]
    fn a_cpu_failure_is_terminal_and_not_retried() {
        let attempts = RefCell::new(Vec::new());
        let result: Result<()> = run_with_fallback(
            TranscriptionDevice::Gpu,
            &AtomicBool::new(false),
            || true,
            |device| {
                attempts.borrow_mut().push(device);
                Err(VeloError::Player("failed".into()))
            },
            |_| {},
        );
        assert!(result.is_err());
        assert_eq!(
            *attempts.borrow(),
            vec![TranscriptionDevice::Gpu, TranscriptionDevice::Cpu]
        );
    }
}
