//! File snapshots and credentials meet only here, never in the chat runner.
use crate::chat::types::*;
use crate::chat::{ChatInput, ChatState};
use crate::commands::settings::StorageState;
use crate::jobs::{JobGate, JobResource};
use crate::storage::settings_store::SummarySettings;
use crate::summary::{Summary, SummaryState, SummaryStore};
use crate::transcript::{Transcript, TranscriptState, TranscriptStore};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub fn canonical_path(path: &str) -> ChatResult<String> {
    let path =
        std::fs::canonicalize(path).map_err(|_| ChatFailure::new(ChatErrorCode::InvalidInput))?;
    if !path.is_file() {
        return Err(ChatFailure::new(ChatErrorCode::InvalidInput));
    }
    path.to_str()
        .map(str::to_string)
        .ok_or_else(|| ChatFailure::new(ChatErrorCode::InvalidInput))
}

pub fn load_snapshot(
    path: &str,
    transcripts: &TranscriptStore,
    summaries: &SummaryStore,
) -> ChatResult<(String, Transcript, Option<Summary>)> {
    let path = canonical_path(path)?;
    let transcript = transcripts
        .load_canonical(&path)
        .filter(|t| t.segments.iter().any(|s| !s.text.trim().is_empty()))
        .ok_or_else(|| ChatFailure::new(ChatErrorCode::MissingTranscript))?;
    let summary = summaries.load_canonical(&path);
    Ok((path, transcript, summary))
}

pub fn status_from_snapshot(
    path: String,
    settings: &SummarySettings,
    has_transcript: bool,
    has_key: bool,
) -> ChatStatus {
    let mut url = reqwest::Url::parse(&settings.base_url)
        .ok()
        .filter(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some());
    let safe = crate::chat::valid_base_url(&settings.base_url);
    if let Some(url) = url.as_mut() {
        let _ = url.set_username("");
        let _ = url.set_password(None);
        url.set_query(None);
        url.set_fragment(None);
    }
    ChatStatus {
        path,
        has_transcript,
        provider: settings.provider.clone(),
        model: settings.model.clone(),
        base_url: url.map(|u| u.to_string()).unwrap_or_default(),
        remote: !crate::summary::is_local(&settings.base_url),
        configured: safe
            && !settings.model.trim().is_empty()
            && matches!(settings.provider.as_str(), "ollama" | "openai" | "deepseek"),
        has_key,
    }
}

#[tauri::command]
pub async fn chat_status(
    path: String,
    transcripts: State<'_, TranscriptState>,
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
) -> ChatResult<ChatStatus> {
    let path = canonical_path(&path)?;
    let settings = storage.settings.load().summary;
    let transcript = transcripts.store.load_canonical(&path);
    let has_key = if crate::chat::valid_base_url(&settings.base_url) {
        summary
            .status(&settings)
            .map_err(|_| ChatFailure::new(ChatErrorCode::NotConfigured))?
            .has_key
    } else {
        false
    };
    Ok(status_from_snapshot(
        path,
        &settings,
        transcript.is_some_and(|t| t.segments.iter().any(|s| !s.text.trim().is_empty())),
        has_key,
    ))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn chat_send(
    app: AppHandle,
    path: String,
    request_id: String,
    message: String,
    history: Vec<ChatHistoryMessage>,
    chat: State<'_, ChatState>,
    transcripts: State<'_, TranscriptState>,
    summary: State<'_, SummaryState>,
    storage: State<'_, StorageState>,
    jobs: State<'_, JobGate>,
) -> ChatResult<ChatStarted> {
    crate::chat::validate_id(&request_id)?;
    // Reserve identity before cache/keychain work, so an early Stop cannot
    // be lost while this synchronous preflight is still preparing a turn.
    let reservation = chat.reserve(&request_id)?;
    let (path, transcript, cached_summary) =
        load_snapshot(&path, &transcripts.store, &summary.store)?;
    let settings = storage.settings.load().summary;
    if !crate::chat::valid_base_url(&settings.base_url) {
        return Err(ChatFailure::new(ChatErrorCode::NotConfigured));
    }
    let language =
        crate::summary::prompt::resolve_language(&settings.language, &transcript.language);
    let api_key = summary
        .load_api_key(&settings)
        .map_err(|_| ChatFailure::new(ChatErrorCode::Auth))?;
    let lease = jobs
        .try_acquire(JobResource::ChatModel, &request_id)
        .map_err(|_| ChatFailure::new(ChatErrorCode::Busy))?;
    reservation.start(
        ChatInput {
            path,
            request_id,
            settings,
            segments: transcript.segments,
            summary: cached_summary.map(|s| s.markdown),
            history,
            message,
            language,
            api_key,
        },
        lease,
        Arc::new(move |event| match event {
            ChatEvent::Delta(payload) => {
                let _ = app.emit("velo://chat-delta", payload);
            }
            ChatEvent::Ready(payload) => {
                let _ = app.emit("velo://chat-ready", payload);
            }
            ChatEvent::Error(payload) => {
                let _ = app.emit("velo://chat-error", payload);
            }
        }),
    )
}

#[tauri::command]
pub async fn chat_cancel(request_id: String, chat: State<'_, ChatState>) -> ChatResult<()> {
    crate::chat::validate_id(&request_id)?;
    chat.cancel(&request_id);
    Ok(())
}
