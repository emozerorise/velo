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
