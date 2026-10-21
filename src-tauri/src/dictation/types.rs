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
