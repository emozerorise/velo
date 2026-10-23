//! Archived extractive experiment, retained for regression comparison only.
use super::{
    evidence::{compose, model::EvidenceModel, source::SourceSnapshot, stages},
    prompt, JobInput, Summary, SummaryStore,
};
use crate::errors::Result;
use std::{
    sync::atomic::AtomicBool,
    time::{SystemTime, UNIX_EPOCH},
};

pub trait EvidenceObserver: Send + Sync {
    fn record(&self, event: &str, payload: &serde_json::Value) -> Result<()>;
}
pub(crate) struct NoopObserver;
impl EvidenceObserver for NoopObserver {
    fn record(&self, _event: &str, _payload: &serde_json::Value) -> Result<()> {
        Ok(())
    }
}
#[cfg(feature = "summary-evidence-qa")]
pub async fn run_observed(
    store: &SummaryStore,
    input: &JobInput,
    cancel: &AtomicBool,
    progress: impl Fn(&str, usize, usize),
    observer: &dyn EvidenceObserver,
) -> Result<Summary> {
    run_inner(store, input, cancel, progress, observer).await
}

pub async fn run(
    store: &SummaryStore,
    input: &JobInput,
    cancel: &AtomicBool,
    progress: impl Fn(&str, usize, usize),
) -> Result<Summary> {
    run_inner(store, input, cancel, progress, &NoopObserver).await
}
async fn run_inner(
    store: &SummaryStore,
    input: &JobInput,
    cancel: &AtomicBool,
    progress: impl Fn(&str, usize, usize),
    observer: &dyn EvidenceObserver,
) -> Result<Summary> {
    let resolved = JobInput {
        settings: crate::storage::settings_store::SummarySettings {
            language: prompt::resolve_language(
                &input.settings.language,
                &input.transcript_language,
            ),
            ..input.settings.clone()
        },
        ..input.clone()
    };
    let sources = SourceSnapshot::new(&resolved.path, &resolved.segments)?;
    let model = EvidenceModel {
        input: &resolved,
        cancel,
        observer,
    };
    model.check_cancel()?;
    progress("extracting_evidence", 0, 1);
    let points = stages::extract_points(&sources, &model).await?;
    observer.record("points", &serde_json::json!({"points":points}))?;
    let plan = stages::plan_points(&sources, &points, &model).await?;
    observer.record(
        "plan",
        &serde_json::json!({"layout":plan.layout(),"occurrences":plan.occurrences()}),
    )?;
    progress("checking_evidence", 0, 1);
    let reviewed = stages::review_points(&sources, &points, plan, &model).await?;
    progress("composing_summary", 0, 1);
    let markdown =
        compose::compose_points(&reviewed, &points, &sources, &resolved.settings.language)?;
    observer.record("composed", &serde_json::json!({"markdown":markdown}))?;
    model.check_cancel()?;
    let summary = Summary {
        meeting_detail: Default::default(),
        format: resolved.settings.format,
        path: resolved.path,
        model: resolved.settings.model,
        language: resolved.settings.language,
        markdown,
        omitted_uncertain_claims: Some(reviewed.omitted()),
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|t| t.as_secs())
            .unwrap_or(0),
        source_segments: sources.segments.len(),
    };
    store.save(&summary)?;
    Ok(summary)
}
