//! Meeting summarisation: fold a transcript into something a person reads.
//!
//! The shape mirrors `transcript`: one job at a time, a cancel flag, and
//! `velo://summary-*` events, so nothing here blocks the UI. Unlike
//! transcription the work is IO-bound rather than CPU-bound, so it runs as a
//! task on Tauri's runtime instead of a dedicated thread.

pub mod chunk;
pub mod credentials;
pub mod prompt;
pub mod store;
pub mod transport;
pub mod types;

pub use store::SummaryStore;
pub use types::{Summary, SummaryDelta, SummaryFailure, SummaryProgress, SummaryStatus};

use crate::errors::{Result, VeloError};
use credentials::ApiKeyStore;

/// Room for a five-section summary of a long meeting, and no more. Anything
/// past this is the model restating itself rather than adding to it.
const ANSWER_TOKENS: u32 = 2_048;
/// Notes on one chunk are working material, and shorter by nature.
const NOTE_TOKENS: u32 = 1_024;
use crate::storage::settings_store::SummarySettings;
use crate::transcript::TranscriptSegment;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

pub struct SummaryState {
    pub store: Arc<SummaryStore>,
    /// The one summary allowed to run at a time, with its cancel flag.
    pub active: Mutex<Option<ActiveJob>>,
    credentials: ApiKeyStore,
}

#[derive(Clone)]
pub struct ActiveJob {
    pub path: String,
    pub cancel: Arc<AtomicBool>,
}

/// Everything a job needs, resolved before it starts so the task owns no
/// borrowed state.
pub struct JobInput {
    pub path: String,
    pub segments: Vec<TranscriptSegment>,
    pub settings: SummarySettings,
    /// The transcript's vocabulary prompt, reused to steer the summariser.
    pub vocabulary: String,
    /// What whisper heard, which is what "auto" resolves to.
    pub transcript_language: String,
    /// Loaded from the keychain in the command layer, never from settings.
    pub api_key: Option<String>,
}

/// True when requests would stay on this machine. Drives the warning the
/// panel shows before anything leaves.
pub fn is_local(base_url: &str) -> bool {
    let host = base_url
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(base_url)
        .split('/')
        .next()
        .unwrap_or("");
    let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);

    matches!(
        host,
        "localhost" | "127.0.0.1" | "::1" | "[::1]" | "0.0.0.0"
    ) || host.ends_with(".localhost")
}

impl SummaryState {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        Ok(Self {
            store: Arc::new(SummaryStore::new(app_handle)?),
            active: Mutex::new(None),
            credentials: ApiKeyStore::default(),
        })
    }

    pub fn status(&self, settings: &SummarySettings) -> Result<SummaryStatus> {
        Ok(SummaryStatus {
            provider: settings.provider.clone(),
            base_url: settings.base_url.clone(),
            model: settings.model.clone(),
            configured: !settings.base_url.trim().is_empty() && !settings.model.trim().is_empty(),
            remote: !is_local(&settings.base_url),
            has_key: self.credentials.has(&settings.base_url)?,
        })
    }

    /// Ask the provider which models it has. Deliberately cheap and quick:
    /// this is the preflight that stops a chained run before an hour of
    /// transcription when the server is not up.
    pub async fn probe(&self, settings: &SummarySettings) -> Result<Vec<String>> {
        let dialect = transport::dialect_for(&settings.provider);
        let key = self.credentials.get(&settings.base_url)?;
        transport::list_models(dialect.as_ref(), &settings.base_url, key.as_deref()).await
    }

    pub fn set_api_key(&self, settings: &SummarySettings, key: &str) -> Result<()> {
        self.credentials.set(&settings.base_url, key)
    }

    pub fn clear_api_key(&self, settings: &SummarySettings) -> Result<()> {
        self.credentials.clear(&settings.base_url)
    }

    pub fn load_api_key(&self, settings: &SummarySettings) -> Result<Option<String>> {
        self.credentials.get(&settings.base_url)
    }

    pub fn cancel(&self) {
        if let Some(job) = self.active.lock().as_ref() {
            job.cancel.store(true, Ordering::Relaxed);
        }
    }

    pub fn start(&self, app: AppHandle, input: JobInput) -> Result<()> {
        if input.segments.is_empty() {
            return Err(VeloError::InvalidParameter(
                "There is no transcript to summarise".into(),
            ));
        }

        let mut active = self.active.lock();
        if let Some(job) = active.as_ref() {
            return Err(VeloError::InvalidParameter(format!(
                "A summary is already being written for {}",
                job.path
            )));
        }

        let cancel = Arc::new(AtomicBool::new(false));
        *active = Some(ActiveJob {
            path: input.path.clone(),
            cancel: cancel.clone(),
        });
        drop(active);

        let store = self.store.clone();

        tauri::async_runtime::spawn(async move {
            let path = input.path.clone();
            let result = run_job(&app, &store, &input, &cancel).await;

            match result {
                Ok(summary) => {
                    info!(
                        "summary ready: {} chars for {}",
                        summary.markdown.len(),
                        path
                    );
                    let _ = app.emit("velo://summary-ready", summary);
                }
                Err(e) => {
                    warn!("summary failed: {}", e);
                    let _ = app.emit(
                        "velo://summary-error",
                        SummaryFailure {
                            path: path.clone(),
                            message: e.to_string(),
                        },
                    );
                }
            }

            if let Some(state) = app_state(&app) {
                *state.active.lock() = None;
            }
        });

        Ok(())
    }
}

fn app_state(app: &AppHandle) -> Option<tauri::State<'_, SummaryState>> {
    use tauri::Manager;
    app.try_state::<SummaryState>()
}

async fn run_job(
    app: &AppHandle,
    store: &SummaryStore,
    input: &JobInput,
    cancel: &Arc<AtomicBool>,
) -> Result<Summary> {
    // "auto" is settled here, once, so every prompt in this job agrees about
    // which language it is asking for.
    let settings = &SummarySettings {
        language: prompt::resolve_language(&input.settings.language, &input.transcript_language),
        ..input.settings.clone()
    };
    let dialect = transport::dialect_for(&settings.provider);
    let budget = chunk::budget_bytes(settings.context_tokens);
    let chunks = chunk::chunk(&input.segments, budget);

    let progress = |stage: &str, done: usize, total: usize| {
        let _ = app.emit(
            "velo://summary-progress",
            SummaryProgress {
                path: input.path.clone(),
                stage: stage.to_string(),
                done,
                total,
            },
        );
    };

    // Streamed straight to the panel, so a long answer is readable as it
    // lands rather than after it finishes.
    let emit_text = |text: &str| {
        let _ = app.emit(
            "velo://summary-delta",
            SummaryDelta {
                path: input.path.clone(),
                text: text.to_string(),
            },
        );
    };

    let markdown = if chunks.len() == 1 {
        // A transcript that already fits needs no notes stage: asking twice
        // would only cost time and lose detail.
        progress("reducing", 0, 1);
        transport::stream_chat(
            dialect.as_ref(),
            &settings.base_url,
            input.api_key.as_deref(),
            &transport::ChatRequest {
                model: settings.model.clone(),
                system: prompt::single_pass_system(settings, &input.vocabulary),
                user: format!("{}{}", chunks[0].text, prompt::transcript_reminder()),
                context_tokens: settings.context_tokens,
                max_tokens: ANSWER_TOKENS,
            },
            cancel,
            emit_text,
            || {},
        )
        .await?
    } else {
        let mut notes = Vec::with_capacity(chunks.len());

        for (index, piece) in chunks.iter().enumerate() {
            progress("mapping", index, chunks.len());

            let note = transport::stream_chat(
                dialect.as_ref(),
                &settings.base_url,
                input.api_key.as_deref(),
                &transport::ChatRequest {
                    model: settings.model.clone(),
                    system: prompt::map_system(settings, &input.vocabulary),
                    user: format!("{}{}", piece.text, prompt::transcript_reminder()),
                    context_tokens: settings.context_tokens,
                    max_tokens: NOTE_TOKENS,
                },
                cancel,
                // The notes are working material; only the final pass is
                // worth putting on screen.
                |_| {},
                || {},
            )
            .await?;

            notes.push(format!(
                "--- [{} – {}] ---\n{}",
                chunk::stamp(piece.start),
                chunk::stamp(piece.end),
                note.trim()
            ));
        }

        progress("mapping", chunks.len(), chunks.len());
        progress("reducing", 0, 1);

        transport::stream_chat(
            dialect.as_ref(),
            &settings.base_url,
            input.api_key.as_deref(),
            &transport::ChatRequest {
                model: settings.model.clone(),
                system: prompt::reduce_system(settings, &input.vocabulary),
                user: format!("{}{}", notes.join("\n\n"), prompt::notes_reminder()),
                context_tokens: settings.context_tokens,
                max_tokens: ANSWER_TOKENS,
            },
            cancel,
            emit_text,
            || {},
        )
        .await?
    };

    let summary = Summary {
        path: input.path.clone(),
        model: settings.model.clone(),
        language: settings.language.clone(),
        markdown: markdown.trim().to_string(),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
        source_segments: input.segments.len(),
    };

    store.save(&summary)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts_are_local() {
        assert!(is_local("http://localhost:11434"));
        assert!(is_local("http://127.0.0.1:11434/v1"));
        assert!(is_local("http://[::1]:11434"));
    }

    #[test]
    fn anything_else_is_remote() {
        assert!(!is_local("https://api.openai.com/v1"));
        assert!(!is_local("http://192.168.1.20:11434"));
        assert!(!is_local("https://localhost.example.com/v1"));
    }
}
