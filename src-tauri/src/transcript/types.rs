//! Serializable transcript types shared with the frontend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSegment {
    /// Seconds from the start of the media, matching the player timeline.
    pub start: f64,
    pub end: f64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcript {
    pub path: String,
    /// Language whisper reported, not the one that was requested.
    pub language: String,
    pub model: String,
    /// The vocabulary prompt this run used, so a disappointing transcript can
    /// be judged against what the model was actually told.
    #[serde(default)]
    pub prompt: String,
    pub created_at: u64,
    pub segments: Vec<TranscriptSegment>,
}

/// Emitted on `velo://transcript-progress` while a job runs.
#[derive(Debug, Clone, Serialize)]
pub struct TranscriptProgress {
    pub path: String,
    /// "extracting" | "transcribing"
    pub stage: String,
    /// 0.0..1.0, or -1.0 when the stage cannot report a fraction.
    pub progress: f64,
}

/// Whether the machine has the pieces the prototype shells out to.
#[derive(Debug, Clone, Serialize)]
pub struct EngineStatus {
    pub whisper_bin: Option<String>,
    pub model_path: Option<String>,
    pub ready: bool,
}
