//! Tauri IPC commands for offline transcription.

use crate::errors::Result;
use crate::transcript::{EngineStatus, Transcript, TranscriptState};
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn transcript_engine_status(state: State<'_, TranscriptState>) -> Result<EngineStatus> {
    Ok(state.engine_status())
}

#[tauri::command]
pub async fn transcript_download_model(
    app: AppHandle,
    state: State<'_, TranscriptState>,
) -> Result<()> {
    state.download_model(app).await
}

#[tauri::command]
pub async fn transcript_cancel_download(state: State<'_, TranscriptState>) -> Result<()> {
    state.cancel_download();
    Ok(())
}

#[tauri::command]
pub async fn transcript_get(
    state: State<'_, TranscriptState>,
    path: String,
) -> Result<Option<Transcript>> {
    Ok(state.store.load(&path))
}

#[tauri::command]
pub async fn transcript_generate(
    app: AppHandle,
    state: State<'_, TranscriptState>,
    path: String,
    language: Option<String>,
    prompt: Option<String>,
) -> Result<()> {
    state.start(
        app,
        path,
        language.unwrap_or_else(|| "auto".into()),
        prompt.unwrap_or_default(),
    )
}

#[tauri::command]
pub async fn transcript_cancel(state: State<'_, TranscriptState>) -> Result<()> {
    state.cancel();
    Ok(())
}

#[tauri::command]
pub async fn transcript_delete(state: State<'_, TranscriptState>, path: String) -> Result<()> {
    state.store.delete(&path)
}
