use serde::Serialize;

pub const MAX_SECONDS: usize = 60;
pub const OUTPUT_RATE: u32 = 16000;

pub struct CapturedAudio {
    pub sample_rate: u32,
    pub mono_samples: Vec<f32>,
}

pub type Result<T> = std::result::Result<T, DictationError>;

/// Stable codes only cross IPC; platform diagnostics may contain private paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum DictationError {
    #[error("Microphone access denied")]
    PermissionDenied,
    #[error("Microphone access restricted")]
    PermissionRestricted,
    #[error("Microphone unavailable")]
    DeviceUnavailable,
    #[error("Unsupported microphone format")]
    UnsupportedFormat,
    #[error("Speech engine busy")]
    EngineBusy,
    #[error("Speech model missing")]
    MissingModel,
    #[error("Microphone capture failed")]
    CaptureFailed,
    #[error("No speech energy detected")]
    Silence,
    #[error("Speech transcription failed")]
    TranscriptionFailed,
    #[error("Dictation cancelled")]
    Cancelled,
    #[error("Invalid dictation input")]
    InvalidInput,
}

pub fn validate_id(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 128 || !id.is_ascii() || id.bytes().any(|b| b.is_ascii_control())
    {
        Err(DictationError::InvalidInput)
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DictationPhase {
    RequestingPermission,
    Recording,
    Transcribing,
}

#[derive(Debug, Clone, Serialize)]
pub struct DictationProgress {
    pub path: String,
    pub session_id: String,
    pub phase: DictationPhase,
    pub elapsed_ms: u64,
    pub paused_playback: bool,
}
#[derive(Debug, Clone, Serialize)]
pub struct DictationReady {
    pub path: String,
    pub session_id: String,
    pub text: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct DictationFailure {
    pub path: String,
    pub session_id: String,
    pub code: DictationError,
    pub cancelled: bool,
}
#[derive(Debug, Clone)]
pub enum DictationEvent {
    State(DictationProgress),
    Ready(DictationReady),
    Error(DictationFailure),
}
