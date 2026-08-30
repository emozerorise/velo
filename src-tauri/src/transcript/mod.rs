//! Offline transcription: extract audio with mpv, run whisper.cpp over it,
//! and cache the timestamped result next to the rest of Velo's state.

pub mod audio;
pub mod engine;
pub mod store;
pub mod types;

pub use store::TranscriptStore;
pub use types::{EngineStatus, Transcript, TranscriptProgress, TranscriptSegment};

use crate::errors::{Result, VeloError};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

pub struct TranscriptState {
    pub store: Arc<TranscriptStore>,
    pub app_data_dir: PathBuf,
    /// The one job allowed to run at a time, with its cancel flag.
    pub active: Mutex<Option<ActiveJob>>,
}

#[derive(Clone)]
pub struct ActiveJob {
    pub path: String,
    pub cancel: Arc<AtomicBool>,
}

/// Serialized in `velo://transcript-error`.
#[derive(serde::Serialize, Clone)]
struct TranscriptError {
    path: String,
    message: String,
}

impl TranscriptState {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        use tauri::Manager;
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| VeloError::Storage(format!("Could not get app data dir: {}", e)))?;

        Ok(Self {
            store: Arc::new(TranscriptStore::new(app_handle)?),
            app_data_dir,
            active: Mutex::new(None),
        })
    }

    pub fn engine_status(&self) -> EngineStatus {
        engine::status(&self.app_data_dir)
    }

    pub fn cancel(&self) {
        if let Some(job) = self.active.lock().as_ref() {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Kick off extraction + transcription on a worker thread. Progress and
    /// the finished transcript arrive as `velo://transcript-*` events, so the
    /// UI never blocks on a job that can run for an hour.
    pub fn start(
        &self,
        app: AppHandle,
        media_path: String,
        language: String,
        prompt: String,
    ) -> Result<()> {
        let mut active = self.active.lock();
        if let Some(job) = active.as_ref() {
            return Err(VeloError::InvalidParameter(format!(
                "A transcription is already running for {}",
                job.path
            )));
        }

        let status = engine::status(&self.app_data_dir);
        let (bin, model) = match (status.whisper_bin, status.model_path) {
            (Some(bin), Some(model)) => (PathBuf::from(bin), PathBuf::from(model)),
            _ => {
                return Err(VeloError::InvalidParameter(
                    "whisper-cli or the model file was not found. Set VELO_WHISPER_BIN and \
                     VELO_WHISPER_MODEL, or drop a ggml .bin into the app's whisper/ folder."
                        .into(),
                ))
            }
        };

        let cancel = Arc::new(AtomicBool::new(false));
        *active = Some(ActiveJob {
            path: media_path.clone(),
            cancel: cancel.clone(),
        });
        drop(active);

        let store = self.store.clone();
        let work_dir = store.work_dir()?;

        std::thread::Builder::new()
            .name("velo-transcribe".into())
            .spawn(move || {
                let wav = work_dir.join("job.wav");
                let result = run_job(
                    &app,
                    &store,
                    &bin,
                    &model,
                    &media_path,
                    &language,
                    &prompt,
                    &wav,
                    &cancel,
                );
                let _ = std::fs::remove_file(&wav);

                match result {
                    Ok(transcript) => {
                        info!(
                            "transcript ready: {} segments for {}",
                            transcript.segments.len(),
                            transcript.path
                        );
                        let _ = app.emit("velo://transcript-ready", transcript);
                    }
                    Err(e) => {
                        warn!("transcription failed: {}", e);
                        let _ = app.emit(
                            "velo://transcript-error",
                            TranscriptError {
                                path: media_path.clone(),
                                message: e.to_string(),
                            },
                        );
                    }
                }

                if let Some(state) = app_state(&app) {
                    *state.active.lock() = None;
                }
            })
            .map_err(|e| VeloError::Player(format!("Could not start transcription: {}", e)))?;

        Ok(())
    }
}

fn app_state(app: &AppHandle) -> Option<tauri::State<'_, TranscriptState>> {
    use tauri::Manager;
    app.try_state::<TranscriptState>()
}

#[allow(clippy::too_many_arguments)]
fn run_job(
    app: &AppHandle,
    store: &TranscriptStore,
    bin: &std::path::Path,
    model: &std::path::Path,
    media_path: &str,
    language: &str,
    prompt: &str,
    wav: &std::path::Path,
    cancel: &Arc<AtomicBool>,
) -> Result<Transcript> {
    let emit = |stage: &str, progress: f64| {
        let _ = app.emit(
            "velo://transcript-progress",
            TranscriptProgress {
                path: media_path.to_string(),
                stage: stage.to_string(),
                progress,
            },
        );
    };

    emit("extracting", -1.0);
    audio::extract(media_path, wav, cancel, &|p| emit("extracting", p))?;

    emit("transcribing", -1.0);
    let (detected, segments) =
        engine::transcribe(bin, model, wav, language, prompt, cancel, &|p| {
            emit("transcribing", p)
        })?;

    let transcript = Transcript {
        path: media_path.to_string(),
        language: detected,
        model: model
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        prompt: prompt.to_string(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        segments,
    };

    store.save(&transcript)?;
    Ok(transcript)
}
