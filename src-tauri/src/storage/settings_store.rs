//! Persistent user settings store with atomic JSON disk writes.

use crate::errors::{Result, VeloError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default)]
    pub font_family: FontFamily,
    pub theme: String, // "dark", "light", "system"
    pub language: String,
    pub remember_playback_position: bool,
    pub auto_play_next: bool,
    #[serde(default = "default_show_summary_timestamps")]
    pub show_summary_timestamps: bool,
}

fn default_show_summary_timestamps() -> bool {
    true
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FontFamily {
    Sarabun,
    IbmPlexSansThaiLooped,
    Prompt,
    Kanit,
    IbmPlexSansThai,
    NotoSansThai,
    #[default]
    #[serde(other)]
    NotoSansThaiLooped,
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
    #[serde(default)]
    pub device: crate::transcript::engine::TranscriptionDevice,
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
            device: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryFormat {
    KeyPoints,
    Custom,
    #[default]
    #[serde(other)]
    Meeting,
}

/// Where summaries come from. `provider` picks the transport and is set
/// explicitly rather than inferred from the URL: LM Studio speaks the OpenAI
/// dialect on loopback, and Ollama can sit behind a remote hostname, so host
/// and dialect are independent facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummarySettings {
    #[serde(default)]
    pub format: SummaryFormat,
    /// "ollama" | "openai" | "deepseek"
    pub provider: String,
    /// The provider's documented base URL, including a version segment
    /// only if required (OpenAI wants /v1; DeepSeek wants the bare root).
    pub base_url: String,
    pub model: String,
    /// "th" | "en" | "auto" -- the language of the summary, which need not
    /// be the language of the meeting or of the app.
    pub language: String,
    /// Free-form steering appended to the prompt.
    pub instructions: String,
    /// Drives chunk sizing, and is sent as `num_ctx` on the Ollama
    /// transport. 32768 is what Ollama was measured serving for both the
    /// old default (qwen3:8b) and the current one, despite the latter
    /// declaring far more.
    pub context_tokens: u32,
}

impl Default for SummarySettings {
    fn default() -> Self {
        Self {
            format: SummaryFormat::Meeting,
            provider: "ollama".into(),
            base_url: "http://localhost:11434".into(),
            model: "qwen3:30b-instruct".into(),
            language: "auto".into(),
            instructions: String::new(),
            context_tokens: 32_768,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UpdateSettings {
    pub auto_check: bool,
    pub last_checked_at: u64,
    pub skipped_version: String,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check: true,
            last_checked_at: 0,
            skipped_version: String::new(),
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
    #[serde(default)]
    pub summary: SummarySettings,
    #[serde(default)]
    pub updates: UpdateSettings,
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
                show_summary_timestamps: true,
                font_family: FontFamily::default(),
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
            summary: SummarySettings::default(),
            updates: UpdateSettings::default(),
        }
    }
}

static SETTINGS_WRITE_LOCK: parking_lot::Mutex<()> = parking_lot::Mutex::new(());

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
        let _write = SETTINGS_WRITE_LOCK.lock();
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
    #[test]
    fn transcript_device_defaults_for_old_settings_and_round_trips_all_modes() {
        use crate::transcript::engine::TranscriptionDevice;
        let mut legacy = serde_json::to_value(super::AppSettings::default()).unwrap();
        legacy["transcript"]
            .as_object_mut()
            .unwrap()
            .remove("device");
        legacy["transcript"]["prompt"] = serde_json::json!("Keep my vocabulary");
        let parsed: super::AppSettings = serde_json::from_value(legacy).unwrap();
        assert_eq!(parsed.transcript.device, TranscriptionDevice::Auto);
        assert_eq!(parsed.transcript.prompt, "Keep my vocabulary");
        for value in ["auto", "gpu", "cpu"] {
            let mut json = serde_json::to_value(&parsed).unwrap();
            json["transcript"]["device"] = serde_json::json!(value);
            let restored: super::AppSettings = serde_json::from_value(json).unwrap();
            assert_eq!(
                serde_json::to_value(restored).unwrap()["transcript"]["device"],
                value
            );
        }
    }

    #[test]
    fn font_preferences_are_backward_compatible_and_round_trip() {
        let directory = std::env::temp_dir().join(format!(
            "velo-font-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let store = super::SettingsStore {
            file_path: directory.join("settings.json"),
        };
        assert_eq!(
            store.load().general.font_family,
            super::FontFamily::NotoSansThaiLooped
        );
        let mut value = serde_json::to_value(super::AppSettings::default()).unwrap();
        value["general"]
            .as_object_mut()
            .unwrap()
            .remove("font_family");
        value["general"]["language"] = serde_json::json!("th");
        for font in [
            None,
            Some("future_font"),
            Some("noto_sans_thai"),
            Some("sarabun"),
            Some("noto_sans_thai_looped"),
            Some("ibm_plex_sans_thai_looped"),
            Some("prompt"),
            Some("kanit"),
            Some("ibm_plex_sans_thai"),
        ] {
            if let Some(font) = font {
                value["general"]["font_family"] = serde_json::json!(font);
            }
            let parsed: super::AppSettings = serde_json::from_value(value.clone()).unwrap();
            store.save(&parsed).unwrap();
            let saved = serde_json::to_value(store.load()).unwrap();
            let expected = match font {
                None | Some("future_font") => "noto_sans_thai_looped",
                Some(font) => font,
            };
            assert_eq!(saved["general"]["font_family"], expected);
            assert_eq!(saved["general"]["language"], "th");
        }
        std::fs::remove_file(directory.join("settings.json")).unwrap();
        std::fs::remove_dir(directory).unwrap();
    }
    use super::*;

    #[test]
    fn summary_format_defaults_without_resetting_existing_preferences() {
        let mut value = serde_json::to_value(AppSettings::default()).unwrap();
        value["summary"]["model"] = serde_json::json!("saved-model");
        value["summary"]["instructions"] = serde_json::json!("Keep the risks");
        value["general"]["language"] = serde_json::json!("th");
        value["summary"].as_object_mut().unwrap().remove("format");
        let settings: AppSettings = serde_json::from_value(value).unwrap();
        let saved = serde_json::to_value(settings).unwrap();
        assert_eq!(saved["summary"]["format"], "meeting");
        assert_eq!(saved["summary"]["model"], "saved-model");
        assert_eq!(saved["summary"]["instructions"], "Keep the risks");
        assert_eq!(saved["general"]["language"], "th");
    }

    #[test]
    fn chosen_summary_formats_survive_settings_round_trip() {
        for (format, expected) in [
            ("meeting", "meeting"),
            ("key_points", "key_points"),
            ("custom", "custom"),
            ("future_format", "meeting"),
        ] {
            let mut value = serde_json::to_value(SummarySettings::default()).unwrap();
            value["format"] = serde_json::json!(format);
            let settings: SummarySettings = serde_json::from_value(value).unwrap();
            assert_eq!(serde_json::to_value(settings).unwrap()["format"], expected);
        }
    }

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
        // Same protection for the section added after summarisation existed.
        assert_eq!(settings.summary.provider, "ollama");
        assert_eq!(settings.summary.model, "qwen3:30b-instruct");
    }

    #[test]
    fn test_settings_keep_summary_section_when_present() {
        let mut original = AppSettings::default();
        original.summary.model = "qwen3:14b".into();
        original.summary.context_tokens = 8192;

        let json = serde_json::to_string(&original).expect("Serialization failed");
        let parsed: AppSettings = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(parsed.summary.model, "qwen3:14b");
        assert_eq!(parsed.summary.context_tokens, 8192);
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

    #[test]
    fn update_preferences_default_without_resetting_older_settings() {
        let mut legacy = serde_json::to_value(AppSettings::default()).expect("serialize settings");
        legacy
            .as_object_mut()
            .expect("settings object")
            .remove("updates");
        legacy["general"]["language"] = serde_json::json!("th");
        let settings: AppSettings = serde_json::from_value(legacy).expect("legacy settings");
        assert_eq!(settings.general.language, "th");
        assert!(settings.updates.auto_check);
        assert_eq!(settings.updates.last_checked_at, 0);
        assert!(settings.updates.skipped_version.is_empty());
    }

    #[test]
    fn legacy_settings_show_summary_timestamps_without_resetting_preferences() {
        let mut legacy = serde_json::to_value(AppSettings::default()).unwrap();
        legacy["general"]
            .as_object_mut()
            .unwrap()
            .remove("show_summary_timestamps");
        legacy["summary"]["instructions"] = serde_json::json!("Keep the agreed owner");
        legacy["general"]["language"] = serde_json::json!("th");
        let parsed: AppSettings = serde_json::from_value(legacy).unwrap();
        let saved = serde_json::to_value(parsed).unwrap();
        assert_eq!(saved["general"]["show_summary_timestamps"], true);
        assert_eq!(saved["general"]["language"], "th");
        assert_eq!(saved["summary"]["instructions"], "Keep the agreed owner");
    }

    #[test]
    fn hidden_summary_timestamps_survive_a_settings_roundtrip() {
        let mut saved = serde_json::to_value(AppSettings::default()).unwrap();
        saved["general"]["show_summary_timestamps"] = serde_json::json!(false);
        let parsed: AppSettings = serde_json::from_value(saved).unwrap();
        assert_eq!(
            serde_json::to_value(parsed).unwrap()["general"]["show_summary_timestamps"],
            false
        );
    }
}
