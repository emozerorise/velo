//! Model/engine/player adapters meet here; dictation never writes source caches.
use crate::commands::settings::StorageState;
#[cfg(not(feature = "dictation-qa"))]
use crate::dictation::{capture::NativeCapture, permission::NativePermission};
use crate::dictation::{types::*, DictationInput, DictationState};
use crate::jobs::JobGate;
use crate::player::PlayerManager;
use crate::transcript::TranscriptState;
#[cfg(not(feature = "dictation-qa"))]
use crate::transcript::{engine, model};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State, WebviewWindow};

#[cfg(feature = "dictation-qa")]
#[path = "../../tests/support/dictation_qa.rs"]
mod qa;

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn dictation_start(
    app: AppHandle,
    window: WebviewWindow,
    path: String,
    session_id: String,
    dictation: State<'_, DictationState>,
    transcript: State<'_, TranscriptState>,
    storage: State<'_, StorageState>,
    jobs: State<'_, JobGate>,
    player: State<'_, Arc<PlayerManager>>,
) -> Result<()> {
    let reservation = dictation.reserve(&session_id)?;
    #[cfg(not(feature = "dictation-qa"))]
    let model = model::resolve(&transcript.app_data_dir).ok_or(DictationError::MissingModel)?;
    #[cfg(feature = "dictation-qa")]
    let adapters = {
        let _ = &transcript;
        qa::adapters(&app.config().identifier)?
    };
    #[cfg(feature = "dictation-qa")]
    let model = adapters.model;
    let path = std::fs::canonicalize(&path).map_err(|_| DictationError::InvalidInput)?;
    if !path.is_file() {
        return Err(DictationError::InvalidInput);
    }
    let path = path
        .to_str()
        .ok_or(DictationError::InvalidInput)?
        .to_string();
    let settings = storage.settings.load().transcript;
    let pause_player = player.inner().clone();
    let check_player = player.inner().clone();
    let expected = path.clone();
    reservation.start(
        DictationInput {
            path,
            session_id,
            model,
            language: settings.language,
            vocabulary: settings.prompt,
            #[cfg(feature = "dictation-qa")]
            engine: adapters.engine,
            #[cfg(not(feature = "dictation-qa"))]
            engine: Arc::new(|model, samples, language, vocabulary, cancel| {
                engine::transcribe(
                    model,
                    samples,
                    language,
                    vocabulary,
                    &cancel.flag(),
                    Arc::new(|_| {}),
                )
                .map(|(_, segments)| {
                    segments
                        .into_iter()
                        .map(|segment| segment.text)
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .map_err(|_| DictationError::TranscriptionFailed)
            }),
        },
        {
            #[cfg(feature = "dictation-qa")]
            {
                adapters.permission
            }
            #[cfg(not(feature = "dictation-qa"))]
            {
                Arc::new(NativePermission)
            }
        },
        {
            #[cfg(feature = "dictation-qa")]
            {
                adapters.capture
            }
            #[cfg(not(feature = "dictation-qa"))]
            {
                Arc::new(NativeCapture)
            }
        },
        Arc::new(move || {
            pause_player
                .pause()
                .map_err(|_| DictationError::CaptureFailed)
        }),
        Arc::new(move || {
            window.is_focused().unwrap_or(false)
                && check_player
                    .current_path()
                    .and_then(|path| std::fs::canonicalize(path).ok())
                    .is_some_and(|path| path.to_str() == Some(expected.as_str()))
        }),
        &jobs,
        Arc::new(move |event| match event {
            DictationEvent::State(event) => {
                let _ = app.emit("velo://dictation-state", event);
            }
            DictationEvent::Ready(event) => {
                let _ = app.emit("velo://dictation-ready", event);
            }
            DictationEvent::Error(event) => {
                let _ = app.emit("velo://dictation-error", event);
            }
        }),
    )
}

#[tauri::command]
pub async fn dictation_stop(
    session_id: String,
    dictation: State<'_, DictationState>,
) -> Result<()> {
    dictation.stop(&session_id)
}
#[tauri::command]
pub async fn dictation_cancel(
    session_id: String,
    dictation: State<'_, DictationState>,
) -> Result<()> {
    dictation.cancel(&session_id)
}
