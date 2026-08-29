use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlaybackState {
    Idle,
    Loading,
    Playing,
    Paused,
    Stopped,
    Ended,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: i64,
    pub src_id: i64,
    #[serde(rename = "type")]
    pub track_type: String, // "video", "audio", "sub"
    pub title: Option<String>,
    pub lang: Option<String>,
    pub codec: Option<String>,
    pub selected: bool,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: Option<String>,
    pub time: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaInfo {
    pub path: String,
    pub file_name: String,
    pub duration: f64,
    pub width: i64,
    pub height: i64,
    pub video_codec: Option<String>,
    pub audio_codec: Option<String>,
    pub fps: f64,
    pub hwdec_current: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeUpdate {
    pub current_time: f64,
    pub duration: f64,
    pub percent: f64,
}
