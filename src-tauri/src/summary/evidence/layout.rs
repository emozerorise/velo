use super::source::SourceSnapshot;
use crate::{
    errors::{Result, VeloError},
    storage::settings_store::SummaryFormat,
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HeadingKind {
    Overview,
    Topics,
    Decisions,
    Actions,
    OpenQuestions,
    KeyPoints,
    Details,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LayoutBlock {
    Heading { heading: HeadingKind },
    Paragraph { claims: Vec<String> },
    List { claims: Vec<String> },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanStatus {
    Supported,
    Unsupported,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutPlan {
    pub job: String,
    pub status: PlanStatus,
    pub blocks: Vec<LayoutBlock>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Occurrence {
    pub id: String,
    pub point_id: String,
    pub heading: Option<HeadingKind>,
    pub block_index: usize,
}
pub struct PointPlan {
    plan: LayoutPlan,
    occurrences: Vec<Occurrence>,
}
impl PointPlan {
    pub fn layout(&self) -> &LayoutPlan {
        &self.plan
    }
    pub fn occurrences(&self) -> &[Occurrence] {
        &self.occurrences
    }
}
pub fn validate_points_plan(
    plan: LayoutPlan,
    points: &[super::types::SourcePoint],
    format: SummaryFormat,
    sources: &SourceSnapshot,
) -> Result<PointPlan> {
    use std::collections::BTreeSet;
    let bad = || VeloError::Summary("Invalid evidence layout; previous summary unchanged".into());
    sources.check_job(&plan.job)?;
    if plan.status == PlanStatus::Unsupported {
        return Err(if plan.blocks.is_empty() {
            VeloError::Summary("unsupported_summary_layout".into())
        } else {
            bad()
        });
    }
    let ids: BTreeSet<_> = points.iter().map(|p| p.id.as_str()).collect();
    if ids.len() != points.len() || ids.contains("") {
        return Err(bad());
    }
    let mut heading = None;
    let mut pending = false;
    let mut overview = BTreeSet::new();
    let mut body = BTreeSet::new();
    let mut occurrences = Vec::new();
    for (block_index, block) in plan.blocks.iter().enumerate() {
        match block {
            LayoutBlock::Heading { heading: h } => {
                let allowed = match format {
                    // Quoting a passage must not promote it to an agreed outcome.
                    SummaryFormat::Meeting => {
                        matches!(h, HeadingKind::Overview | HeadingKind::Topics)
                    }
                    SummaryFormat::KeyPoints => {
                        matches!(h, HeadingKind::Overview | HeadingKind::KeyPoints)
                    }
                    SummaryFormat::Custom => matches!(
                        h,
                        HeadingKind::Overview | HeadingKind::KeyPoints | HeadingKind::Details
                    ),
                };
                if pending || !allowed {
                    return Err(bad());
                }
                heading = Some(*h);
                pending = true;
            }
            LayoutBlock::Paragraph { claims } | LayoutBlock::List { claims } => {
                if claims.is_empty() || (format != SummaryFormat::Custom && heading.is_none()) {
                    return Err(bad());
                }
                for id in claims {
                    let selected = if heading == Some(HeadingKind::Overview) {
                        &mut overview
                    } else {
                        &mut body
                    };
                    if !ids.contains(id.as_str()) || !selected.insert(id.clone()) {
                        return Err(bad());
                    }
                    occurrences.push(Occurrence {
                        id: format!("O{:06}", occurrences.len() + 1),
                        point_id: id.clone(),
                        heading,
                        block_index,
                    });
                }
                pending = false;
            }
        }
    }
    if pending || occurrences.is_empty() {
        return Err(bad());
    }
    Ok(PointPlan { plan, occurrences })
}
