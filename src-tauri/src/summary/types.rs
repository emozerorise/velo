//! Serializable summary types shared with the frontend.

use crate::storage::settings_store::SummaryFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Kept with the result so changing the next-run preference cannot
    /// reorder an existing custom summary. Older caches are meetings.
    #[serde(default)]
    pub format: SummaryFormat,
    pub path: String,
    /// The model that wrote it, so a disappointing summary can be judged
    /// against what produced it.
    pub model: String,
    pub language: String,
    /// Application-composed Markdown; legacy model-written caches remain readable.
    pub markdown: String,
    /// Absence on an older cache is not a certification of its contents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omitted_uncertain_claims: Option<usize>,
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
    /// "mapping" | "reducing" | "reviewing"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_cached_summaries_keep_their_text_and_default_to_meetings() {
        let summary: Summary = serde_json::from_str(
            r###"{
            "path":"/clip.wav","model":"saved-model","language":"th",
            "markdown":"## ภาพรวม\nข้อความเดิม", "created_at":42,"source_segments":8
        }"###,
        )
        .unwrap();
        let saved = serde_json::to_value(summary).unwrap();
        assert_eq!(saved["format"], "meeting");
        assert_eq!(saved["markdown"], "## ภาพรวม\nข้อความเดิม");
    }
}
