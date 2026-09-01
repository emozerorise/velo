//! Serializable summary types shared with the frontend.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub path: String,
    /// The model that wrote it, so a disappointing summary can be judged
    /// against what produced it.
    pub model: String,
    pub language: String,
    /// Markdown, stored exactly as the model wrote it. Timestamp citations
    /// are linkified at render time rather than parsed here.
    pub markdown: String,
    pub created_at: u64,
    /// Segment count of the transcript this was made from. A mismatch with
    /// the current transcript is what marks a summary stale.
    pub source_segments: usize,
}

/// What the provider settings currently describe. Reachability is a separate
/// question, answered by `summary_probe`.
#[derive(Debug, Clone, Serialize)]
pub struct SummaryStatus {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    /// Both a base URL and a model are present.
    pub configured: bool,
    /// The request would leave this machine.
    pub remote: bool,
    /// Whether the current destination host has a key in the OS keychain.
    pub has_key: bool,
}

/// Emitted on `velo://summary-progress`.
#[derive(Debug, Clone, Serialize)]
pub struct SummaryProgress {
    pub path: String,
    /// "mapping" | "reducing"
    pub stage: String,
    pub done: usize,
    pub total: usize,
}

/// Emitted on `velo://summary-delta` as the final pass streams in.
#[derive(Debug, Clone, Serialize)]
pub struct SummaryDelta {
    pub path: String,
    pub text: String,
}

/// Emitted on `velo://summary-error`.
#[derive(Debug, Clone, Serialize)]
pub struct SummaryFailure {
    pub path: String,
    pub message: String,
}
