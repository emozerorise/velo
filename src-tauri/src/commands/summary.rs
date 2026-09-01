//! Tauri IPC commands for meeting summarisation.
//!
//! This is the only place the summary and transcript modules meet: the
//! summariser needs segments and the transcript's vocabulary, and gets both
//! handed to it here rather than reaching across for them.

use crate::commands::settings::StorageState;
use crate::errors::{Result, VeloError};
use crate::summary::{JobInput, Summary, SummaryState, SummaryStatus};
use crate::transcript::TranscriptState;
use tauri::{AppHandle, State};

#[tauri::command]
pub async fn summary_status(
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
) -> Result<SummaryStatus> {
    summary.status(&storage.settings.load().summary)
}

/// The models the provider actually has. Used as the preflight before a
/// chained run, and to fill the model picker in settings.
#[tauri::command]
pub async fn summary_probe(
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
) -> Result<Vec<String>> {
    let settings = storage.settings.load().summary;
    summary.probe(&settings).await
}

#[tauri::command]
pub async fn summary_set_api_key(
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
    key: String,
) -> Result<()> {
    summary.set_api_key(&storage.settings.load().summary, &key)
}

#[tauri::command]
pub async fn summary_clear_api_key(
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
) -> Result<()> {
    summary.clear_api_key(&storage.settings.load().summary)
}

#[tauri::command]
pub async fn summary_get(
    summary: State<'_, SummaryState>,
    path: String,
) -> Result<Option<Summary>> {
    Ok(summary.store.load(&path))
}

#[tauri::command]
pub async fn summary_generate(
    app: AppHandle,
    summary: State<'_, SummaryState>,
    transcripts: State<'_, TranscriptState>,
    storage: State<'_, StorageState>,
    path: String,
) -> Result<()> {
    let settings = storage.settings.load();
    let api_key = summary.load_api_key(&settings.summary)?;

    let transcript = transcripts.store.load(&path).ok_or_else(|| {
        VeloError::InvalidParameter("There is no transcript for this file yet".into())
    })?;

    summary.start(
        app,
        JobInput {
            path,
            transcript_language: transcript.language,
            segments: transcript.segments,
            settings: settings.summary,
            vocabulary: settings.transcript.prompt,
            api_key,
        },
    )
}

#[tauri::command]
pub async fn summary_cancel(summary: State<'_, SummaryState>) -> Result<()> {
    summary.cancel();
    Ok(())
}

#[tauri::command]
pub async fn summary_delete(summary: State<'_, SummaryState>, path: String) -> Result<()> {
    summary.store.delete(&path)
}
