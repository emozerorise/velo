//! Persistent user settings store with atomic JSON disk writes.

use crate::errors::{Result, VeloError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    pub theme: String, // "dark", "light", "system"
    pub language: String,
    pub remember_playback_position: bool,
    pub auto_play_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoSettings {
    pub hardware_acceleration: bool,
    pub default_aspect_ratio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    pub default_volume: f64,
    pub preferred_language: String,
    pub volume_step: f64,
    pub audio_delay_step: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleSettings {
    pub preferred_language: String,
    pub auto_load_external: bool,
    pub font_size: u32,
    pub subtitle_delay_step: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSettings {
    pub language: String,
    /// Domain vocabulary handed to whisper as its initial prompt. Meetings
    /// that mix Thai with English product terms transcribe far better when
    /// the model has seen those terms spelled correctly first.
    pub prompt: String,
}

impl Default for TranscriptSettings {
    fn default() -> Self {
        Self {
            language: "auto".into(),
            prompt: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: u32,
    pub general: GeneralSettings,
    pub video: VideoSettings,
    pub audio: AudioSettings,
    pub subtitle: SubtitleSettings,
    /// Defaulted so a settings.json written before transcription existed
    /// still loads instead of silently resetting every other preference.
    #[serde(default)]
    pub transcript: TranscriptSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 1,
            general: GeneralSettings {
                theme: "dark".into(),
                language: "en".into(),
                remember_playback_position: true,
                auto_play_next: true,
            },
            video: VideoSettings {
                hardware_acceleration: true,
                default_aspect_ratio: "auto".into(),
            },
            audio: AudioSettings {
                default_volume: 80.0,
                preferred_language: "eng".into(),
                volume_step: 5.0,
                audio_delay_step: 0.1,
            },
            subtitle: SubtitleSettings {
                preferred_language: "eng".into(),
                auto_load_external: true,
                font_size: 48,
                subtitle_delay_step: 0.1,
            },
            transcript: TranscriptSettings::default(),
        }
    }
}

pub struct SettingsStore {
    file_path: PathBuf,
}

impl SettingsStore {
    pub fn new(app_handle: &tauri::AppHandle) -> Result<Self> {
        let app_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| VeloError::Storage(format!("Could not get config dir: {}", e)))?;

        fs::create_dir_all(&app_dir)
            .map_err(|e| VeloError::Storage(format!("Could not create config dir: {}", e)))?;

        let file_path = app_dir.join("settings.json");
        Ok(Self { file_path })
    }

    pub fn load(&self) -> AppSettings {
        if self.file_path.exists() {
            if let Ok(content) = fs::read_to_string(&self.file_path) {
                if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
                    return settings;
                }
            }
        }
        AppSettings::default()
    }

    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        let json = serde_json::to_string_pretty(settings)
            .map_err(|e| VeloError::Storage(format!("Failed to serialize settings: {}", e)))?;

        let tmp_path = self.file_path.with_extension("tmp");
        fs::write(&tmp_path, json)
            .map_err(|e| VeloError::Storage(format!("Failed to write tmp settings: {}", e)))?;

        fs::rename(&tmp_path, &self.file_path)
            .map_err(|e| VeloError::Storage(format!("Failed to save settings: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.version, 1);
        assert_eq!(settings.general.theme, "dark");
        assert!(settings.video.hardware_acceleration);
        assert_eq!(settings.audio.default_volume, 80.0);
    }

    #[test]
    fn test_settings_load_without_transcript_section() {
        // Settings written by an older build must keep their values.
        let legacy = serde_json::json!({
            "version": 1,
            "general": {
                "theme": "light",
                "language": "th",
                "remember_playback_position": false,
                "auto_play_next": false
            },
            "video": { "hardware_acceleration": true, "default_aspect_ratio": "auto" },
            "audio": {
                "default_volume": 42.0,
                "preferred_language": "tha",
                "volume_step": 5.0,
                "audio_delay_step": 0.1
            },
            "subtitle": {
                "preferred_language": "tha",
                "auto_load_external": true,
                "font_size": 48,
                "subtitle_delay_step": 0.1
            }
        });

        let settings: AppSettings =
            serde_json::from_value(legacy).expect("legacy settings should still parse");
        assert_eq!(settings.general.theme, "light");
        assert_eq!(settings.audio.default_volume, 42.0);
        assert_eq!(settings.transcript.language, "auto");
        assert!(settings.transcript.prompt.is_empty());
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let original = AppSettings::default();
        let json = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: AppSettings =
            serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(deserialized.version, original.version);
        assert_eq!(deserialized.general.theme, original.general.theme);
    }
}
