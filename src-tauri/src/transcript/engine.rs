//! In-process transcription with whisper.cpp.
//!
//! whisper is linked into the binary rather than shelled out to, so a release
//! only needs the model file, which `super::model` fetches on first use.

use crate::errors::{Result, VeloError};
use crate::transcript::types::TranscriptSegment;
use std::ffi::c_void;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Once;
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
    transcribe_chunked(
        model,
        samples,
        language,
        prompt,
        cancel,
        on_progress,
        CHUNK_SECONDS,
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
    if samples.is_empty() {
        return Err(VeloError::Player("There is no audio to transcribe".into()));
    }

    // Without this, ggml and whisper write their own diagnostics straight to
    // stderr, outside the app's tracing setup.
    LOGGING_HOOK.call_once(whisper_rs::install_logging_hooks);

    let ctx = WhisperContext::new_with_params(
        model
            .to_str()
            .ok_or_else(|| VeloError::InvalidParameter("Non-UTF8 model path".into()))?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| VeloError::Player(format!("Could not load the whisper model: {}", e)))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| VeloError::Player(format!("Could not start whisper: {}", e)))?;

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
