//! Per-media summary cache, one JSON file per source path.
//!
//! Deliberately a sibling of the transcript rather than a field inside it: a
//! summary is cheap to redo and a transcript is not, so regenerating or
//! deleting one must never rewrite the other.

use crate::errors::{Result, VeloError};
use crate::summary::types::Summary;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tauri::Manager;

pub struct SummaryStore {
    dir: PathBuf,
}

impl SummaryStore {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| VeloError::Storage(format!("Could not get app data dir: {}", e)))?
            .join("summaries");

        fs::create_dir_all(&dir)
            .map_err(|e| VeloError::Storage(format!("Could not create summary dir: {}", e)))?;

        Ok(Self { dir })
    }

    /// Same naming as `TranscriptStore`: a readable stem plus a hash of the
    /// full path, so two files with the same name in different folders do
    /// not collide.
    fn file_for(&self, media_path: &str) -> PathBuf {
        let mut hasher = DefaultHasher::new();
        media_path.hash(&mut hasher);
        let hash = hasher.finish();

        let stem = Path::new(media_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("media")
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .take(40)
            .collect::<String>();

        self.dir.join(format!("{}-{:016x}.json", stem, hash))
    }

    pub fn load(&self, media_path: &str) -> Option<Summary> {
        let content = fs::read_to_string(self.file_for(media_path)).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, summary: &Summary) -> Result<()> {
        let file = self.file_for(&summary.path);
        let json = serde_json::to_string_pretty(summary)
            .map_err(|e| VeloError::Storage(format!("Failed to serialize summary: {}", e)))?;

        let tmp = file.with_extension("tmp");
        fs::write(&tmp, json)
            .map_err(|e| VeloError::Storage(format!("Failed to write tmp summary: {}", e)))?;
        fs::rename(&tmp, &file)
            .map_err(|e| VeloError::Storage(format!("Failed to save summary: {}", e)))?;
        Ok(())
    }

    pub fn delete(&self, media_path: &str) -> Result<()> {
        let file = self.file_for(media_path);
        if file.exists() {
            fs::remove_file(&file)
                .map_err(|e| VeloError::Storage(format!("Failed to delete summary: {}", e)))?;
        }
        Ok(())
    }
}
