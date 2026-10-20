//! Tauri IPC commands for offline transcription.

use crate::errors::Result;
use crate::jobs::{JobGate, JobResource};
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

/// Returns the bytes freed, so the UI can report what it recovered.
#[tauri::command]
pub async fn transcript_remove_model(
    state: State<'_, TranscriptState>,
    jobs: State<'_, JobGate>,
) -> Result<u64> {
    let _lease = jobs.try_acquire(JobResource::Whisper, "remove-model")?;
    state.remove_model()
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
    jobs: State<'_, JobGate>,
    path: String,
    language: Option<String>,
    prompt: Option<String>,
) -> Result<()> {
    state.start(
        app,
        path,
        language.unwrap_or_else(|| "auto".into()),
        prompt.unwrap_or_default(),
        jobs.try_acquire(JobResource::Whisper, "transcript")?,
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
