use crate::summary::transport::WireMessage;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum UserOrAssistant {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistoryMessage {
    pub role: UserOrAssistant,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatSource {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FullOrExcerpts {
    Full,
    Excerpts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FullTruncatedOrAbsent {
    Full,
    Truncated,
    Absent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCoverage {
    pub transcript: FullOrExcerpts,
    pub summary: FullTruncatedOrAbsent,
    pub omitted_pairs: usize,
}

#[derive(Debug, Clone)]
pub struct ChatContext {
    pub messages: Vec<WireMessage>,
    pub sources: Vec<ChatSource>,
    pub coverage: ChatCoverage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChatErrorCode {
    NotConfigured,
    MissingTranscript,
    Auth,
    Connection,
    RateLimit,
    Provider,
    Context,
    Timeout,
    Empty,
    Incomplete,
    Busy,
    Cancelled,
    InvalidInput,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatFailure {
    pub code: ChatErrorCode,
    pub message: String,
}
pub type ChatResult<T> = std::result::Result<T, ChatFailure>;

impl ChatFailure {
    pub fn new(code: ChatErrorCode) -> Self {
        Self {
            code,
            message: match code {
                ChatErrorCode::NotConfigured => "Configure a text model before sending",
                ChatErrorCode::MissingTranscript => "A transcript is required for this file",
                ChatErrorCode::Auth => "The API key was rejected",
                ChatErrorCode::Connection => "Could not reach the model server",
                ChatErrorCode::RateLimit => "The provider is rate limiting this request",
                ChatErrorCode::Context => "The context budget cannot hold this request",
                ChatErrorCode::Timeout => "The model server took too long to answer",
                ChatErrorCode::Empty => "The model returned no answer",
                ChatErrorCode::Incomplete => "The provider stream ended before completion",
                ChatErrorCode::Busy => "The model is busy",
                ChatErrorCode::Cancelled => "Cancelled",
                ChatErrorCode::InvalidInput => "Invalid chat input",
                ChatErrorCode::Provider => "The provider could not complete this request",
            }
            .into(),
        }
    }
}
impl std::fmt::Display for ChatFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for ChatFailure {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStatus {
    pub path: String,
    pub has_transcript: bool,
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub remote: bool,
    pub configured: bool,
    pub has_key: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatStarted {
    pub path: String,
    pub request_id: String,
    pub sources: Vec<ChatSource>,
    pub coverage: ChatCoverage,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChatDelta {
    pub path: String,
    pub request_id: String,
    pub sequence: u64,
    pub text: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChatAnswer {
    pub path: String,
    pub request_id: String,
    pub text: String,
    pub sources: Vec<ChatSource>,
    pub coverage: ChatCoverage,
}
#[derive(Debug, Clone, Serialize)]
pub struct ChatError {
    pub path: String,
    pub request_id: String,
    pub code: ChatErrorCode,
    pub message: String,
    pub cancelled: bool,
}
#[derive(Debug, Clone)]
pub enum ChatEvent {
    Delta(ChatDelta),
    Ready(ChatAnswer),
    Error(ChatError),
}
