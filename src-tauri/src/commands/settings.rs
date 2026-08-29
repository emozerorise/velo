//! Tauri IPC commands for settings and history persistence.

use crate::errors::Result;
use crate::storage::history_store::{HistoryData, PlaybackHistoryItem};
use crate::storage::settings_store::AppSettings;
use crate::storage::{HistoryStore, SettingsStore};
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::State;

pub struct StorageState {
    pub settings: Arc<SettingsStore>,
    pub history: Arc<HistoryStore>,
    pub cached_history: Mutex<HistoryData>,
}

#[tauri::command]
pub async fn settings_get_all(storage: State<'_, StorageState>) -> Result<AppSettings> {
    Ok(storage.settings.load())
}

#[tauri::command]
pub async fn settings_save(storage: State<'_, StorageState>, settings: AppSettings) -> Result<()> {
    storage.settings.save(&settings)
}

#[tauri::command]
pub async fn history_get(storage: State<'_, StorageState>) -> Result<HistoryData> {
    Ok(storage.cached_history.lock().clone())
}

#[tauri::command]
pub async fn history_record(
    storage: State<'_, StorageState>,
    item: PlaybackHistoryItem,
) -> Result<()> {
    let mut history = storage.cached_history.lock();
    history.recent_files.retain(|f| f.path != item.path);
    history.recent_files.insert(0, item.clone());
    if history.recent_files.len() > 25 {
        history.recent_files.truncate(25);
    }
    history
        .resume_positions
        .insert(item.path, item.last_position);
    storage.history.save(&history)
}
