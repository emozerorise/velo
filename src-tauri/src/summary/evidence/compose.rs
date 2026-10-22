use super::source::SourceSnapshot;
use super::{
    layout::{HeadingKind, LayoutBlock},
    types::*,
};
use crate::errors::{Result, VeloError};
pub fn compose_points(
    reviewed: &super::admit::ReviewedPointPlan,
    points: &[SourcePoint],
    sources: &SourceSnapshot,
    language: &str,
) -> Result<String> {
    let bad = || {
        VeloError::Summary("Invalid or empty reviewed summary; previous summary unchanged".into())
    };
    let mut blocks = Vec::new();
    let mut pending = None;
    let mut substantive = false;
    for (index, block) in reviewed.plan().layout().blocks.iter().enumerate() {
        if let LayoutBlock::Heading { heading } = block {
            pending = Some(*heading);
            continue;
        }
        let mut lines = Vec::new();
        for occurrence in reviewed
            .plan()
            .occurrences()
            .iter()
            .filter(|o| o.block_index == index)
        {
            let check = reviewed.check(&occurrence.id).ok_or_else(bad)?;
            if check.verdict != super::admit::Verdict::Supported {
                continue;
            }
            let point = &points
                .iter()
                .find(|p| p.id == occurrence.point_id)
                .ok_or_else(bad)?
                .point;
            let supplied = point
                .evidence
                .iter()
                .chain(&point.required)
                .map(|a| a.source.clone())
                .collect();
            super::source::validate_point(sources, point, &supplied)?;
            let mut text = literal(&point.text);
            // Source prose beginning with an ordered-list marker remains prose.
            if let Some((prefix, _)) = text.split_once(". ") {
                if !prefix.trim().is_empty() && prefix.trim().bytes().all(|c| c.is_ascii_digit()) {
                    text.insert(prefix.len(), '\\');
                }
            }
            let ids: std::collections::BTreeSet<_> = point
                .evidence
                .iter()
                .chain(&point.required)
                .chain(&check.evidence)
                .map(|a| a.source.as_str())
                .collect();
            for id in ids {
                let clock = super::super::validation::source_clock(id, &sources.segments)
                    .ok_or_else(bad)?;
                text.push_str(&format!(" [{clock}]"));
            }
            lines.push(text);
        }
        if lines.is_empty() {
            continue;
        }
        if let Some(h) = pending.take() {
            blocks.push(format!("## {}", heading(h, language == "th")));
        }
        substantive = true;
        blocks.push(if matches!(block, LayoutBlock::List { .. }) {
            lines
                .into_iter()
                .map(|line| format!("- {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            lines.join(" ")
        });
    }
    if !substantive {
        return Err(bad());
    }
    Ok(blocks.join("\n\n"))
}

fn literal(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        if ['\\', '[', ']', '*', '_', '#', '-', '+', '`', '>'].contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(if ch == '\n' || ch == '\r' { ' ' } else { ch });
    }
    escaped
}
pub(super) fn heading(h: HeadingKind, th: bool) -> &'static str {
    match (h, th) {
        (HeadingKind::Overview, true) => "ภาพรวม",
        (HeadingKind::Topics, true) => "ประเด็นที่คุยกัน",
        (HeadingKind::Decisions, true) => "สิ่งที่ตัดสินใจ",
        (HeadingKind::Actions, true) => "งานที่ต้องทำ",
        (HeadingKind::OpenQuestions, true) => "เรื่องที่ยังค้าง",
        (HeadingKind::KeyPoints, true) => "ประเด็นสำคัญ",
        (HeadingKind::Details, true) => "รายละเอียด",
        (HeadingKind::Overview, false) => "Overview",
        (HeadingKind::Topics, false) => "Topics discussed",
        (HeadingKind::Decisions, false) => "Decisions",
        (HeadingKind::Actions, false) => "Action items",
        (HeadingKind::OpenQuestions, false) => "Open questions",
        (HeadingKind::KeyPoints, false) => "Key points",
        (HeadingKind::Details, false) => "Details",
    }
}

#[cfg(test)]
mod point_tests {
    use super::super::{
        admit::{self, OccurrenceCheck, PointReviewReport, Verdict},
        layout::{self, LayoutPlan, PlanStatus},
        source::validate_point,
    };
    use super::*;
    use crate::{storage::settings_store::SummaryFormat, transcript::TranscriptSegment};
    use std::collections::BTreeSet;
    fn a(id: &str, quote: &str) -> Anchor {
        Anchor {
            source: id.into(),
            quote: quote.into(),
        }
    }
    fn fixture() -> (SourceSnapshot, Vec<SourcePoint>) {
        let sources = SourceSnapshot::new(
            "/test.wav",
            &[
                TranscriptSegment {
                    start: 10.25,
                    end: 12.75,
                    text: "When inspection finishes, remove the item.".into(),
                },
                TranscriptSegment {
                    start: 12.75,
                    end: 15.0,
                    text: "Other reviewers no longer see it.".into(),
                },
            ],
        )
        .unwrap();
        let points = vec![SourcePoint {
            id: "P000001".into(),
            point: DraftPoint {
                text:
                    "When inspection finishes, remove the item; other reviewers no longer see it."
                        .into(),
                evidence: vec![
                    a("S000001", "When inspection finishes, remove the item."),
                    a("S000002", "Other reviewers no longer see it."),
                ],
                required: vec![a("S000001", "When inspection finishes")],
            },
        }];
        (sources, points)
    }
    fn plan(s: &SourceSnapshot, headings: &[HeadingKind]) -> LayoutPlan {
        let mut blocks = Vec::new();
        for h in headings {
            blocks.extend([
                LayoutBlock::Heading { heading: *h },
                LayoutBlock::Paragraph {
                    claims: vec!["P000001".into()],
                },
            ]);
        }
        if blocks.is_empty() {
            blocks.push(LayoutBlock::Paragraph {
                claims: vec!["P000001".into()],
            });
        }
        LayoutPlan {
            job: s.job().into(),
            status: PlanStatus::Supported,
            blocks,
        }
    }
    fn report(s: &SourceSnapshot, p: &[SourcePoint], verdicts: &[Verdict]) -> PointReviewReport {
        PointReviewReport {
            job: s.job().into(),
            need_sources: vec![],
            checks: verdicts
                .iter()
                .enumerate()
                .map(|(i, v)| OccurrenceCheck {
                    occurrence: format!("O{:06}", i + 1),
                    verdict: *v,
                    evidence: p[0].point.evidence.clone(),
                })
                .collect(),
        }
    }
    #[test]
    fn whole_point_goldens_preserve_trigger_and_no_status_in_every_format() {
        let (s, p) = fixture();
        for (format, headings, expected) in [
            (SummaryFormat::Meeting, vec![HeadingKind::Overview], "## Overview\n\nWhen inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]"),
            (SummaryFormat::KeyPoints, vec![HeadingKind::KeyPoints], "## Key points\n\nWhen inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]"),
            (SummaryFormat::Custom, vec![], "When inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]"),
        ] {
            let plan = layout::validate_points_plan(plan(&s,&headings), &p, format, &s).unwrap();
            assert_eq!(plan.occurrences()[0].id,"O000001");
            let reviewed = admit::reviewed_plan(&s,&p,plan,report(&s,&p,&[Verdict::Supported])).unwrap();
            assert_eq!(compose_points(&reviewed,&p,&s,"en").unwrap(),expected);
        }
    }
    #[test]
    fn one_point_has_separate_heading_verdicts_and_unique_omission_count() {
        let (s, mut p) = fixture();
        p.push(SourcePoint {
            id: "P000002".into(),
            point: p[0].point.clone(),
        });
        for (verdicts, omitted, expected) in [
            ([Verdict::Supported,Verdict::Unsupported],0,Some("## Overview\n\nWhen inspection finishes, remove the item; other reviewers no longer see it. [00:10] [00:12]")),
            ([Verdict::Unclear,Verdict::Unsupported],1,None),
        ] {
            let plan = layout::validate_points_plan(plan(&s,&[HeadingKind::Overview,HeadingKind::Topics]), &p, SummaryFormat::Meeting, &s).unwrap();
            assert_eq!(plan.occurrences().len(),2);
            assert_eq!(plan.occurrences()[1].heading,Some(HeadingKind::Topics));
            let reviewed = admit::reviewed_plan(&s,&p,plan,report(&s,&p,&verdicts)).unwrap();
            assert_eq!(reviewed.omitted(),omitted);
            let composed = compose_points(&reviewed,&p,&s,"en");
            match expected { Some(text) => assert_eq!(composed.unwrap(),text), None => assert!(composed.is_err()) }
        }
    }
    #[test]
    fn invalid_occurrence_reports_never_get_a_reviewed_plan() {
        let (s, p) = fixture();
        let plan =
            layout::validate_points_plan(plan(&s, &[]), &p, SummaryFormat::Custom, &s).unwrap();
        let valid = report(&s, &p, &[Verdict::Supported]);
        let mut bads = vec![];
        let mut v = valid.clone();
        v.checks.clear();
        bads.push(v);
        let mut v = valid.clone();
        v.checks.push(v.checks[0].clone());
        bads.push(v);
        let mut v = valid.clone();
        v.checks[0].occurrence = "O999999".into();
        bads.push(v);
        let mut v = valid.clone();
        v.job = "other-job".into();
        bads.push(v);
        let mut v = valid.clone();
        v.checks[0].evidence.clear();
        bads.push(v);
        let mut v = valid.clone();
        v.checks[0].evidence[0].quote = "invented".into();
        bads.push(v);
        let mut v = valid.clone();
        v.checks[0].evidence.remove(0);
        bads.push(v);
        let mut v = valid.clone();
        v.need_sources.push("S000002".into());
        bads.push(v);
        for v in bads {
            assert!(admit::validate_occurrence_report(&s, &p, plan.occurrences(), &v).is_err());
        }
        assert!(admit::validate_occurrence_report(&s, &p, plan.occurrences(), &valid).is_ok());
        let mut old = serde_json::to_value(valid).unwrap();
        old["checks"][0]["fields"] = serde_json::json!([]);
        assert!(serde_json::from_value::<PointReviewReport>(old).is_err());
    }
    #[test]
    fn layout_rejects_fragment_ids_duplicates_and_outcome_headings_in_all_formats() {
        let (s, p) = fixture();
        let mut bad = plan(&s, &[]);
        bad.blocks.push(bad.blocks[0].clone());
        assert!(layout::validate_points_plan(bad, &p, SummaryFormat::Custom, &s).is_err());
        let mut bad = plan(&s, &[]);
        bad.blocks[0] = LayoutBlock::Paragraph {
            claims: vec!["P000001:effect".into()],
        };
        assert!(layout::validate_points_plan(bad, &p, SummaryFormat::Custom, &s).is_err());
        for format in [
            SummaryFormat::Custom,
            SummaryFormat::KeyPoints,
            SummaryFormat::Meeting,
        ] {
            for heading in [
                HeadingKind::Decisions,
                HeadingKind::Actions,
                HeadingKind::OpenQuestions,
            ] {
                assert!(
                    layout::validate_points_plan(plan(&s, &[heading]), &p, format, &s).is_err()
                );
            }
        }
        let unsupported = LayoutPlan {
            job: s.job().into(),
            status: PlanStatus::Unsupported,
            blocks: vec![],
        };
        assert_eq!(
            layout::validate_points_plan(unsupported, &p, SummaryFormat::Custom, &s)
                .err()
                .unwrap()
                .to_string(),
            "unsupported_summary_layout"
        );
        for extra in [
            serde_json::json!({"kind":"heading","heading":"The feature exists"}),
            serde_json::json!({"kind":"paragraph","claims":["P000001"],"text":"replacement"}),
        ] {
            assert!(serde_json::from_value::<LayoutBlock>(extra).is_err());
        }
    }
    #[test]
    fn translated_framing_preserves_thai_negation_uncertainty_and_literal_source_clocks() {
        let raw = "เมื่อตรวจเสร็จ อาจไม่แสดง ## heading [00:10]";
        let s = SourceSnapshot::new(
            "/thai.wav",
            &[TranscriptSegment {
                start: 12.75,
                end: 15.0,
                text: raw.into(),
            }],
        )
        .unwrap();
        let p = vec![SourcePoint {
            id: "P000001".into(),
            point: DraftPoint {
                text: format!("They asked whether {raw}"),
                evidence: vec![a("S000001", raw)],
                required: vec![a("S000001", "เมื่อตรวจเสร็จ"), a("S000001", "อาจไม่แสดง")],
            },
        }];
        let plan =
            layout::validate_points_plan(plan(&s, &[]), &p, SummaryFormat::Custom, &s).unwrap();
        let reviewed =
            admit::reviewed_plan(&s, &p, plan, report(&s, &p, &[Verdict::Supported])).unwrap();
        assert_eq!(
            compose_points(&reviewed, &p, &s, "en").unwrap(),
            "They asked whether เมื่อตรวจเสร็จ อาจไม่แสดง \\#\\# heading \\[00:10\\] [00:12]"
        );
        let mut changed = p[0].point.clone();
        changed.text = changed.text.replace("อาจไม่แสดง", "แสดง");
        assert!(validate_point(&s, &changed, &BTreeSet::from(["S000001".into()])).is_err());
    }
    #[test]
    fn answered_question_is_removed_whole_and_cannot_be_relabelled() {
        let s = SourceSnapshot::new(
            "/question.wav",
            &[
                TranscriptSegment {
                    start: 10.25,
                    end: 12.75,
                    text: "Which mode?".into(),
                },
                TranscriptSegment {
                    start: 150.25,
                    end: 153.0,
                    text: "Use groups; that answers the earlier question.".into(),
                },
            ],
        )
        .unwrap();
        let p = vec![SourcePoint {
            id: "P000001".into(),
            point: DraftPoint {
                text: "Which mode remains unresolved?".into(),
                evidence: vec![a("S000001", "Which mode?")],
                required: vec![],
            },
        }];
        let plan = layout::validate_points_plan(
            plan(&s, &[HeadingKind::Topics]),
            &p,
            SummaryFormat::Meeting,
            &s,
        )
        .unwrap();
        let mut rejected = report(&s, &p, &[Verdict::Unsupported]);
        rejected.checks[0].evidence = vec![a(
            "S000002",
            "Use groups; that answers the earlier question.",
        )];
        let reviewed = admit::reviewed_plan(&s, &p, plan, rejected).unwrap();
        assert_eq!(reviewed.omitted(), 1);
        assert!(compose_points(&reviewed, &p, &s, "en").is_err());
    }
    #[test]
    fn source_ordered_list_marker_is_literal_not_structure() {
        let s = SourceSnapshot::new(
            "/literal.wav",
            &[TranscriptSegment {
                start: 12.75,
                end: 15.0,
                text: "1. Literal heading [00:10]".into(),
            }],
        )
        .unwrap();
        let p = vec![SourcePoint {
            id: "P000001".into(),
            point: DraftPoint {
                text: s.segments[0].text.clone(),
                evidence: vec![a("S000001", &s.segments[0].text)],
                required: vec![],
            },
        }];
        let plan =
            layout::validate_points_plan(plan(&s, &[]), &p, SummaryFormat::Custom, &s).unwrap();
        let reviewed =
            admit::reviewed_plan(&s, &p, plan, report(&s, &p, &[Verdict::Supported])).unwrap();
        assert_eq!(
            compose_points(&reviewed, &p, &s, "en").unwrap(),
            "1\\. Literal heading \\[00:10\\] [00:12]"
        );
    }
}
