//! Meeting summarisation: fold a transcript into something a person reads.
//!
//! The shape mirrors `transcript`: one job at a time, a cancel flag, and
//! `velo://summary-*` events, so nothing here blocks the UI. Unlike
//! transcription the work is IO-bound rather than CPU-bound, so it runs as a
//! task on Tauri's runtime instead of a dedicated thread.

#[cfg(test)]
mod audit;
pub mod chunk;
pub mod credentials;
mod evidence;
pub mod evidence_pipeline;
mod generation;
mod parts;
pub mod pipeline;
pub mod prompt;
pub mod store;
pub mod transport;
pub mod types;
mod validation;

pub use store::SummaryStore;
pub use types::{Summary, SummaryDelta, SummaryFailure, SummaryProgress, SummaryStatus};

use crate::errors::{Result, VeloError};
use crate::jobs::JobLease;
use credentials::ApiKeyStore;

/// The local model's measured limits prevent repetitive, runaway output.
/// DeepSeek gets twice the room without changing existing providers.
const ANSWER_TOKENS: u32 = 2_048;
/// Notes on one chunk are working material, and shorter by nature.
const NOTE_TOKENS: u32 = 1_024;
use crate::storage::settings_store::SummarySettings;
use crate::transcript::TranscriptSegment;
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
#[derive(Clone)]
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
            credentials: ApiKeyStore::for_app(&app_handle.config().identifier),
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

    pub fn start(&self, app: AppHandle, input: JobInput, lease: JobLease) -> Result<()> {
        if input.segments.is_empty() {
            return Err(VeloError::InvalidParameter(
                "There is no transcript to summarise".into(),
            ));
        }

        let mut active = self.active.lock();
        if active.is_some() {
            return Err(VeloError::InvalidParameter(
                "A summary is already being written".into(),
            ));
        }

        let cancel = Arc::new(AtomicBool::new(false));
        *active = Some(ActiveJob {
            path: input.path.clone(),
            cancel: cancel.clone(),
        });
        drop(active);

        let store = self.store.clone();
        let cleanup = SummaryCleanup {
            app: app.clone(),
            cancel: cancel.clone(),
        };

        tauri::async_runtime::spawn(async move {
            let _lease = lease;
            let _cleanup = cleanup;
            let path = input.path.clone();
            let result = run_job(&app, &store, &input, &cancel).await;

            match result {
                Ok(summary) => {
                    info!("summary ready");
                    let _ = app.emit("velo://summary-ready", summary);
                }
                Err(e) => {
                    warn!("summary failed");
                    let _ = app.emit(
                        "velo://summary-error",
                        SummaryFailure {
                            path: path.clone(),
                            message: e.to_string(),
                        },
                    );
                }
            }
        });

        Ok(())
    }
}

struct SummaryCleanup {
    app: AppHandle,
    cancel: Arc<AtomicBool>,
}
impl Drop for SummaryCleanup {
    fn drop(&mut self) {
        if let Some(state) = app_state(&self.app) {
            let mut active = state.active.lock();
            if active
                .as_ref()
                .is_some_and(|job| Arc::ptr_eq(&job.cancel, &self.cancel))
            {
                *active = None;
            }
        }
    }
}

fn app_state(app: &AppHandle) -> Option<tauri::State<'_, SummaryState>> {
    use tauri::Manager;
    app.try_state::<SummaryState>()
}

fn completion_request(
    settings: &SummarySettings,
    system: String,
    user: String,
    notes: bool,
) -> transport::ChatRequest {
    transport::ChatRequest {
        model: settings.model.clone(),
        system,
        user,
        context_tokens: settings.context_tokens,
        max_tokens: match (settings.provider.as_str(), notes) {
            ("deepseek", true) => 2_048,
            ("deepseek", false) => 4_096,
            (_, true) => NOTE_TOKENS,
            (_, false) => ANSWER_TOKENS,
        },
    }
}

async fn run_job(
    app: &AppHandle,
    store: &SummaryStore,
    input: &JobInput,
    cancel: &Arc<AtomicBool>,
) -> Result<Summary> {
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

    pipeline::run(store, input, cancel, progress).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deepseek_gets_room_for_notes_and_a_complete_summary_without_changing_local_limits() {
        for (provider, notes, expected) in [
            ("deepseek", false, 4_096),
            ("deepseek", true, 2_048),
            ("ollama", false, 2_048),
            ("ollama", true, 1_024),
            ("openai", false, 2_048),
            ("openai", true, 1_024),
        ] {
            let settings = SummarySettings {
                provider: provider.into(),
                ..Default::default()
            };
            let req =
                completion_request(&settings, "instructions".into(), "transcript".into(), notes);
            assert_eq!(req.max_tokens, expected, "{provider} notes={notes}");
            assert_eq!(req.system, "instructions");
            assert_eq!(req.user, "transcript");
        }
    }

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
