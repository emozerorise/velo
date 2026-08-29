//! Playlist commands for queue and folder ingestion.

use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub files: Vec<String>,
}

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "mov", "avi", "webm", "flv", "m4v", "ts", "wmv", "mpg", "mpeg", "vob",
];

#[tauri::command]
pub async fn playlist_scan_directory(dir_path: String) -> Result<ScanResult> {
    let mut files = Vec::new();
    let path = Path::new(&dir_path);
    if path.is_dir() {
        scan_dir_recursive(path, &mut files);
    }
    files.sort();
    Ok(ScanResult { files })
}

fn scan_dir_recursive(dir: &Path, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir_recursive(&path, files);
            } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                    if let Some(p_str) = path.to_str() {
                        files.push(p_str.to_string());
                    }
                }
            }
        }
    }
}
