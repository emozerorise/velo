use super::{source::SourceSnapshot, types::*};
use crate::errors::{Result, VeloError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Supported,
    Unsupported,
    Unclear,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OccurrenceCheck {
    pub occurrence: String,
    pub verdict: Verdict,
    pub evidence: Vec<Anchor>,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PointReviewReport {
    pub job: String,
    pub checks: Vec<OccurrenceCheck>,
    pub need_sources: Vec<String>,
}
pub struct ReviewedPointPlan {
    plan: super::layout::PointPlan,
    checks: Vec<OccurrenceCheck>,
    omitted: usize,
}
impl ReviewedPointPlan {
    pub fn plan(&self) -> &super::layout::PointPlan {
        &self.plan
    }
    pub fn check(&self, occurrence: &str) -> Option<&OccurrenceCheck> {
        self.checks
            .iter()
            .find(|check| check.occurrence == occurrence)
    }
    pub fn omitted(&self) -> usize {
        self.omitted
    }
}
pub fn validate_occurrence_report(
    sources: &SourceSnapshot,
    points: &[SourcePoint],
    occurrences: &[super::layout::Occurrence],
    report: &PointReviewReport,
) -> Result<()> {
    sources.check_job(&report.job)?;
    if !report.need_sources.is_empty() || report.checks.len() != occurrences.len() {
        return Err(invalid());
    }
    let mut seen = std::collections::BTreeSet::new();
    for check in &report.checks {
        let occurrence = occurrences
            .iter()
            .find(|o| o.id == check.occurrence)
            .ok_or_else(invalid)?;
        let point = &points
            .iter()
            .find(|p| p.id == occurrence.point_id)
            .ok_or_else(invalid)?
            .point;
        if !seen.insert(&check.occurrence) {
            return Err(invalid());
        }
        let mut anchors = std::collections::BTreeSet::new();
        for a in &check.evidence {
            sources.resolve(a)?;
            if !anchors.insert((&a.source, &a.quote)) {
                return Err(invalid());
            }
        }
        if check.verdict == Verdict::Supported
            && point.evidence.iter().chain(&point.required).any(|a| {
                !check
                    .evidence
                    .iter()
                    .any(|e| e.source == a.source && e.quote.contains(&a.quote))
            })
        {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn reviewed_plan(
    sources: &SourceSnapshot,
    points: &[SourcePoint],
    plan: super::layout::PointPlan,
    report: PointReviewReport,
) -> Result<ReviewedPointPlan> {
    validate_occurrence_report(sources, points, plan.occurrences(), &report)?;
    let mut selected = std::collections::BTreeSet::new();
    let mut supported = std::collections::BTreeSet::new();
    for o in plan.occurrences() {
        selected.insert(&o.point_id);
        if report
            .checks
            .iter()
            .any(|c| c.occurrence == o.id && c.verdict == Verdict::Supported)
        {
            supported.insert(&o.point_id);
        }
    }
    let omitted = selected.difference(&supported).count();
    Ok(ReviewedPointPlan {
        plan,
        checks: report.checks,
        omitted,
    })
}

fn invalid() -> VeloError {
    VeloError::Summary("Invalid placement report; previous summary unchanged".into())
}
