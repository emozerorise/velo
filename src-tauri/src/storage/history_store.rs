//! Recent playback history & resume playback position store.

use crate::errors::{Result, VeloError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackHistoryItem {
    pub path: String,
    pub file_name: String,
    pub last_position: f64,
    pub duration: f64,
    pub last_played_timestamp: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HistoryData {
    pub recent_files: Vec<PlaybackHistoryItem>,
    pub resume_positions: HashMap<String, f64>,
}

pub struct HistoryStore {
    file_path: PathBuf,
}

impl HistoryStore {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| VeloError::Storage(format!("Could not get app data dir: {}", e)))?;

        fs::create_dir_all(&app_dir)
            .map_err(|e| VeloError::Storage(format!("Could not create app data dir: {}", e)))?;

        let file_path = app_dir.join("history.json");
        Ok(Self { file_path })
    }

    pub fn load(&self) -> HistoryData {
        if self.file_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(data) = serde_json::from_str::<HistoryData>(&content) {
                    return data;
                }
            }
        }
        HistoryData::default()
    }

    pub fn save(&self, data: &HistoryData) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| VeloError::Storage(format!("Failed to serialize history: {}", e)))?;

        let tmp_path = self.file_path.with_extension("tmp");
        fs::write(&tmp_path, json)
            .map_err(|e| VeloError::Storage(format!("Failed to write tmp history: {}", e)))?;

        fs::rename(&tmp_path, &self.file_path)
            .map_err(|e| VeloError::Storage(format!("Failed to save history: {}", e)))?;

        Ok(())
    }
}
