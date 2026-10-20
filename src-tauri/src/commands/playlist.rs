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
    "mp4", "mkv", "mov", "avi", "webm", "flv", "m4v", "ts", "wmv", "mpg", "mpeg", "vob", "mp3",
    "wav", "m4a", "aac", "flac", "ogg", "opus",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn scans_audio_and_video_in_nested_folders_and_skips_other_files() {
        let id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("velo-media-scan-{id}"));
        fs::create_dir_all(dir.join("nested")).expect("create fixture directory");

        let media = [
            "meeting.mp3",
            "meeting.MP4",
            "nested/voice.WAV",
            "nested/voice.m4a",
            "nested/voice.aac",
            "nested/voice.flac",
            "nested/voice.ogg",
            "nested/voice.opus",
        ];
        for name in media
            .iter()
            .chain(["notes.txt", "cover.jpg", "no-extension"].iter())
        {
            fs::write(dir.join(name), []).expect("create fixture file");
        }

        let result = playlist_scan_directory(dir.to_string_lossy().into_owned()).await;
        fs::remove_dir_all(&dir).expect("remove fixture directory");

        let mut expected: Vec<String> = media
            .iter()
            .map(|name| dir.join(name).to_string_lossy().into_owned())
            .collect();
        expected.sort();
        assert_eq!(result.expect("scan fixture directory").files, expected);
    }
}
