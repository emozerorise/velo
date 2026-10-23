use super::{
    admit::{self, PointReviewReport, ReviewedPointPlan, Verdict},
    extraction,
    layout::{self, LayoutPlan, Occurrence, PointPlan},
    model::EvidenceModel,
    prompts,
    source::SourceSnapshot,
    types::SourcePoint,
};
use crate::errors::{Result, VeloError};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
#[cfg(test)]
#[allow(dead_code)]
#[path = "../../../tests/support/evidence_fixtures.rs"]
mod wire;

const REPAIR_SCHEMA:&str="Previous output failed strict schema validation. Return the complete current-job JSON with exactly the allowed keys.";
const REPAIR_JOB:&str="Previous output used the wrong job token. Copy the exact current job from this request, retaining the strict schema.";
const REPAIR_LAYOUT:&str="Previous layout failed whole-ID, heading, format or duplicate-placement validation. Return the complete corrected layout.";
const REPAIR_REPORT:&str="Previous report failed complete unique occurrence coverage, supplied exact evidence or required-phrase coverage. Return the complete corrected report.";
fn invalid() -> VeloError {
    VeloError::Summary(
        "Evidence validation failed after bounded repair; previous summary unchanged".into(),
    )
}
fn context() -> VeloError {
    VeloError::Summary("Evidence exceeds context budget; previous summary unchanged".into())
}
fn reserved_fits(model: &EvidenceModel<'_>, system: &str, body: &Value) -> bool {
    [REPAIR_SCHEMA, REPAIR_JOB, REPAIR_LAYOUT, REPAIR_REPORT]
        .iter()
        .all(|feedback| {
            let mut reserved = body.clone();
            reserved["repair"] = json!(feedback);
            model.fits(system, &reserved.to_string())
        })
}
fn raw(sources: &SourceSnapshot, ids: &[String], neighbours: bool) -> Result<Value> {
    serde_json::from_str(&sources.lines(ids, neighbours)?).map_err(|_| invalid())
}
fn supplied(rows: &Value) -> BTreeSet<String> {
    rows.as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| row["source"].as_str().map(str::to_owned))
        .collect()
}
pub async fn extract_points(
    sources: &SourceSnapshot,
    model: &EvidenceModel<'_>,
) -> Result<Vec<SourcePoint>> {
    let system = prompts::points(
        &model.input.settings,
        &model.input.vocabulary,
        sources.job(),
    );
    let body = |rows: Value| json!({"job":sources.job(),"sources":rows,"repair":"","reminder":"Select supplied source IDs only; the app copies whole original passages. Group complete conditions/questions and their effects. Never generate or correct wording."});
    let ranges = sources.packet_ranges(|rows| {
        serde_json::from_str::<Value>(rows).is_ok_and(|rows| {
            let payload = body(rows);
            model.fits(&system, &payload.to_string())
                && extraction::feedback_reservations()
                    .into_iter()
                    .all(|feedback| {
                        let mut reserved = payload.clone();
                        reserved["repair"] = feedback;
                        model.fits(&system, &reserved.to_string())
                    })
        })
    })?;
    let mut points = Vec::new();
    let mut unique = BTreeSet::new();
    for range in ranges {
        model.check_cancel()?;
        let ids: Vec<_> = range.map(super::super::chunk::source_id).collect();
        let rows = raw(sources, &ids, false)?;
        let allowed = supplied(&rows);
        let mut payload = body(rows);
        let mut accepted = None;
        for _ in 0..2 {
            let answer = model
                .complete("extracting", system.clone(), payload.to_string())
                .await?;
            match extraction::decode_packet(&answer, sources, &allowed) {
                Err(failure) => payload["repair"] = failure.feedback(),
                Ok(points) => {
                    accepted = Some(points);
                    break;
                }
            }
        }
        for point in accepted.ok_or_else(invalid)? {
            let shape = serde_json::to_string(&point).map_err(|_| invalid())?;
            if unique.insert(shape) {
                points.push(SourcePoint {
                    id: format!("P{:06}", points.len() + 1),
                    point,
                });
            }
        }
    }
    if points.is_empty() {
        return Err(invalid());
    }
    Ok(points)
}
pub async fn plan_points(
    sources: &SourceSnapshot,
    points: &[SourcePoint],
    model: &EvidenceModel<'_>,
) -> Result<PointPlan> {
    let system = prompts::points_layout(&model.input.settings, sources.job());
    // The arranger needs the visible quotations once, not three copies in anchors.
    let catalog: Vec<_> = points
        .iter()
        .map(|p| json!({"id":p.id,"text":p.point.text,"words":super::length::words(&p.point.text)}))
        .collect();
    let limit = super::length::ceiling(&model.input.settings);
    let mut body = json!({"job":sources.job(),"points":catalog,"format":model.input.settings.format,"max_words":limit,"repair":"","reminder":prompts::layout_contract(&model.input.settings)});
    if !reserved_fits(model, &system, &body) {
        return Err(context());
    }
    if limit.is_some() {
        let mut reserved = body.clone();
        reserved["repair"] = length_feedback(usize::MAX, usize::MAX);
        if !model.fits(&system, &reserved.to_string()) {
            return Err(context());
        }
    }
    for _ in 0..2 {
        let answer = model
            .complete("planning", system.clone(), body.to_string())
            .await?;
        let plan = match serde_json::from_str::<LayoutPlan>(&answer) {
            Ok(p) => p,
            Err(_) => {
                body["repair"] = json!(REPAIR_SCHEMA);
                continue;
            }
        };
        if sources.check_job(&plan.job).is_err() {
            body["repair"] = json!(REPAIR_JOB);
            continue;
        }
        match layout::validate_points_plan(plan, points, model.input.settings.format, sources) {
            Ok(plan) => {
                let count: usize = plan
                    .layout()
                    .blocks
                    .iter()
                    .map(|block| match block {
                        layout::LayoutBlock::Heading { heading } => {
                            super::length::words(super::compose::heading(
                                *heading,
                                model.input.settings.language == "th",
                            ))
                        }
                        layout::LayoutBlock::Paragraph { claims }
                        | layout::LayoutBlock::List { claims } => claims
                            .iter()
                            .map(|id| {
                                points
                                    .iter()
                                    .find(|p| &p.id == id)
                                    .map_or(0, |p| super::length::words(&p.point.text))
                            })
                            .sum(),
                    })
                    .sum();
                if let Some(limit) = limit.filter(|&limit| count > limit) {
                    body["repair"] = length_feedback(count, limit);
                } else {
                    return Ok(plan);
                }
            }
            Err(e) if e.to_string() == "unsupported_summary_layout" => return Err(e),
            Err(_) => body["repair"] = json!(REPAIR_LAYOUT),
        }
    }
    Err(invalid())
}
fn length_feedback(actual: usize, limit: usize) -> Value {
    json!({"category":"word_limit_exceeded","actual":actual,"limit":limit,"instruction":"Select a complete corrected layout within max_words using the supplied per-point word counts. Choose shorter whole excerpts for the same main themes, including later material. Never cut a condition, alter quotations or silently drop a main theme. The app will not truncate or shorten text."})
}
fn point_ids(points: &[SourcePoint], occurrences: &[Occurrence]) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for o in occurrences {
        let point = &points
            .iter()
            .find(|p| p.id == o.point_id)
            .ok_or_else(invalid)?
            .point;
        ids.extend(
            point
                .evidence
                .iter()
                .chain(&point.required)
                .map(|a| a.source.clone()),
        );
    }
    Ok(ids)
}
fn occurrence_rows(points: &[SourcePoint], occurrences: &[Occurrence]) -> Result<Vec<Value>> {
    occurrences.iter().map(|o|{
        let point=&points.iter().find(|p|p.id==o.point_id).ok_or_else(invalid)?.point;
        Ok(json!({"id":o.id,"point_id":o.point_id,"heading":o.heading,"block_index":o.block_index,"text":point.text,"evidence":point.evidence,"required":point.required}))
    }).collect()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceReport {
    job: String,
    checks: Vec<SourceCheck>,
    need_sources: Vec<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceCheck {
    occurrence: String,
    verdict: Verdict,
    sources: Vec<String>,
}
fn source_report(answer: &str, sources: &SourceSnapshot) -> Result<PointReviewReport> {
    let packet: SourceReport = serde_json::from_str(answer).map_err(|_| invalid())?;
    let mut checks = Vec::new();
    for check in packet.checks {
        let mut seen = BTreeSet::new();
        let mut evidence = Vec::new();
        for id in check.sources {
            if !seen.insert(id.clone()) {
                return Err(invalid());
            }
            let rows = raw(sources, std::slice::from_ref(&id), false)?;
            let quote = rows[0]["text"].as_str().ok_or_else(invalid)?.to_string();
            evidence.push(super::types::Anchor { source: id, quote });
        }
        checks.push(admit::OccurrenceCheck {
            occurrence: check.occurrence,
            verdict: check.verdict,
            evidence,
        });
    }
    Ok(PointReviewReport {
        job: packet.job,
        checks,
        need_sources: packet.need_sources,
    })
}
pub async fn review_points(
    sources: &SourceSnapshot,
    points: &[SourcePoint],
    plan: PointPlan,
    model: &EvidenceModel<'_>,
) -> Result<ReviewedPointPlan> {
    let system = prompts::placement_review(sources.job());
    let catalog:Vec<_>=points.iter().map(|p|json!({"id":p.id,"text":p.point.text,"sources":p.point.evidence.iter().map(|a|a.source.as_str()).collect::<BTreeSet<_>>()})).collect();
    let all_ids: Vec<_> = (0..sources.segments.len())
        .map(super::super::chunk::source_id)
        .collect();
    let all_raw = raw(sources, &all_ids, false)?;
    let mut checks = Vec::new();
    let mut begin = 0;
    while begin < plan.occurrences().len() {
        let mut end = (begin + 4).min(plan.occurrences().len());
        let mut body;
        loop {
            let batch = &plan.occurrences()[begin..end];
            body = json!({"job":sources.job(),"occurrences":occurrence_rows(points,batch)?,"sources":all_raw,"catalog":catalog,"follow_up":false,"repair":"","reminder":"Check the complete visible text AND heading. Exact raw evidence, required wording and later answers matter; reject whole unsupported occurrences."});
            if reserved_fits(model, &system, &body) {
                break;
            }
            body["sources"] = raw(sources, &point_ids(points, batch)?, true)?;
            if reserved_fits(model, &system, &body) {
                break;
            }
            if end == begin + 1 {
                return Err(context());
            }
            end -= 1;
        }
        let batch = &plan.occurrences()[begin..end];
        let mut accepted = None;
        for attempt in 0..2 {
            model.check_cancel()?;
            let answer = model
                .complete("checking", system.clone(), body.to_string())
                .await?;
            let mut report = match source_report(&answer, sources) {
                Ok(r) => r,
                Err(_) => {
                    body["repair"] = json!(REPAIR_SCHEMA);
                    continue;
                }
            };
            if sources.check_job(&report.job).is_err() {
                body["repair"] = json!(REPAIR_JOB);
                continue;
            }
            let wanted = std::mem::take(&mut report.need_sources);
            let allowed = supplied(&body["sources"]);
            if admit::validate_occurrence_report(sources, points, batch, &report).is_err()
                || report
                    .checks
                    .iter()
                    .any(|c| c.evidence.iter().any(|a| !allowed.contains(&a.source)))
            {
                body["repair"] = json!(REPAIR_REPORT);
                continue;
            }
            if !wanted.is_empty() {
                // Validate even an exhausted request; invented source IDs are not an unclear verdict.
                let fetched = raw(sources, &wanted, true)?;
                if attempt == 0 {
                    let mut ids: Vec<_> = allowed.into_iter().collect();
                    ids.extend(supplied(&fetched));
                    body["sources"] = raw(sources, &ids, false)?;
                    body["follow_up"] = json!(true);
                    if !reserved_fits(model, &system, &body) {
                        return Err(context());
                    }
                    continue;
                }
                for check in &mut report.checks {
                    check.verdict = Verdict::Unclear;
                }
            }
            model.observer.record(
                "review",
                &serde_json::to_value(&report).map_err(|_| invalid())?,
            )?;
            accepted = Some(report.checks);
            break;
        }
        checks.extend(accepted.ok_or_else(invalid)?);
        begin = end;
    }
    admit::reviewed_plan(
        sources,
        points,
        plan,
        PointReviewReport {
            job: sources.job().into(),
            checks,
            need_sources: vec![],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::wire;
    use super::*;
    use crate::{
        storage::settings_store::{SummaryFormat, SummarySettings},
        summary::JobInput,
        transcript::TranscriptSegment,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    fn input(base_url: String, segments: Vec<TranscriptSegment>) -> JobInput {
        JobInput {
            path: "/synthetic.wav".into(),
            segments,
            settings: SummarySettings {
                provider: "deepseek".into(),
                base_url,
                language: "en".into(),
                context_tokens: 8192,
                ..Default::default()
            },
            transcript_language: "en".into(),
            vocabulary: String::new(),
            api_key: None,
        }
    }
    fn segment(i: usize, text: String) -> TranscriptSegment {
        TranscriptSegment {
            start: i as f64 * 5.0,
            end: i as f64 * 5.0 + 4.0,
            text,
        }
    }
    fn model<'a>(job: &'a JobInput, cancel: &'a AtomicBool) -> EvidenceModel<'a> {
        EvidenceModel {
            input: job,
            cancel,
            observer: &super::super::super::evidence_pipeline::NoopObserver,
        }
    }
    #[tokio::test]
    async fn extraction_overlap_deduplicates_only_identical_text_and_anchors() {
        let mut packet = wire::point_packet();
        packet["points"][0]["sources"][0] = json!("S000011");
        let mut last = packet.clone();
        last["points"][0]["sources"][0] = json!("S000021");
        let split = packet.clone();
        let (base, requests) = wire::server(vec![packet, split, last]);
        let job = input(
            base,
            (0..25)
                .map(|i| segment(i, "Please add a group option.".into()))
                .collect(),
        );
        let s = SourceSnapshot::new(&job.path, &job.segments).unwrap();
        let cancel = AtomicBool::new(false);
        let points = extract_points(&s, &model(&job, &cancel)).await.unwrap();
        assert_eq!(points.len(), 2);
        assert_eq!(points[0].id, "P000001");
        assert_eq!(points[1].id, "P000002");
        assert_eq!(points[1].point.evidence[0].source, "S000021");
        assert_eq!(requests.iter().count(), 3);
    }
    #[tokio::test]
    async fn extraction_rejects_real_but_unsupplied_packet_source() {
        let (base, requests) = wire::server(vec![
            wire::point_packet(),
            wire::point_packet(),
            wire::point_packet(),
        ]);
        let job = input(
            base,
            (0..25)
                .map(|i| segment(i, "Please add a group option.".into()))
                .collect(),
        );
        let s = SourceSnapshot::new(&job.path, &job.segments).unwrap();
        let cancel = AtomicBool::new(false);
        assert!(extract_points(&s, &model(&job, &cancel)).await.is_err());
        assert_eq!(requests.iter().count(), 3);
    }
    #[tokio::test]
    async fn distant_answer_is_fetched_as_raw_context_before_rejecting_unresolved_claim() {
        let mut segments = (0..30)
            .map(|i| segment(i, format!("Unrelated source {i}: {}", "x".repeat(900))))
            .collect::<Vec<_>>();
        segments[0].text = "Which mode?".into();
        segments[29].text = "Use groups; this answers the earlier question.".into();
        let mut follow = json!({"job":"$JOB","checks":[{"occurrence":"O000001","verdict":"unclear","sources":[]}],"need_sources":["S000030"]});
        let first = follow.clone();
        follow["need_sources"] = json!([]);
        follow["checks"][0]["verdict"] = json!("unsupported");
        follow["checks"][0]["sources"] = json!(["S000030"]);
        let (base, requests) = wire::server(vec![first, follow]);
        let job = input(base, segments);
        let s = SourceSnapshot::new(&job.path, &job.segments).unwrap();
        let cancel = AtomicBool::new(false);
        let points = vec![SourcePoint {
            id: "P000001".into(),
            point: super::super::types::DraftPoint {
                text: "Which mode remains unresolved?".into(),
                evidence: vec![super::super::types::Anchor {
                    source: "S000001".into(),
                    quote: "Which mode?".into(),
                }],
                required: vec![],
            },
        }];
        let plan=layout::validate_points_plan(serde_json::from_value(json!({"job":s.job(),"status":"supported","blocks":[{"kind":"heading","heading":"topics"},{"kind":"paragraph","claims":["P000001"]}]})).unwrap(),&points,SummaryFormat::Meeting,&s).unwrap();
        let reviewed = review_points(&s, &points, plan, &model(&job, &cancel))
            .await
            .unwrap();
        assert_eq!(reviewed.omitted(), 1);
        let requests: Vec<_> = requests.iter().collect();
        assert_eq!(requests.len(), 2);
        let body = |i: usize| {
            serde_json::from_str::<Value>(requests[i]["messages"][1]["content"].as_str().unwrap())
                .unwrap()
        };
        assert!(!body(0)["sources"]
            .to_string()
            .contains("Use groups; this answers"));
        assert!(body(1)["sources"]
            .to_string()
            .contains("Use groups; this answers the earlier question."));
        assert!(body(1)["sources"]
            .to_string()
            .contains("Unrelated source 28"));
    }
    #[tokio::test]
    async fn indivisible_point_catalog_or_layout_overflow_fails_before_http() {
        let job = input(
            "http://127.0.0.1:1".into(),
            vec![segment(0, "Please add a group option.".into())],
        );
        let s = SourceSnapshot::new(&job.path, &job.segments).unwrap();
        let cancel = AtomicBool::new(false);
        let model = model(&job, &cancel);
        let mut point: super::super::types::DraftPoint =
            serde_json::from_value(wire::canonical_point()).unwrap();
        point.text = "x".repeat(30000);
        let points = vec![SourcePoint {
            id: "P000001".into(),
            point,
        }];
        assert!(plan_points(&s, &points, &model)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("context"));
        let mut plan = wire::point_plan();
        plan["job"] = json!(s.job());
        let plan = layout::validate_points_plan(
            serde_json::from_value(plan).unwrap(),
            &points,
            SummaryFormat::Meeting,
            &s,
        )
        .unwrap();
        assert!(review_points(&s, &points, plan, &model)
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("context"));
    }

    #[tokio::test]
    async fn extraction_reserves_every_feedback_before_http() {
        struct Requests(AtomicUsize);
        impl crate::summary::evidence_pipeline::EvidenceObserver for Requests {
            fn record(&self, event: &str, _payload: &Value) -> Result<()> {
                if event == "request" {
                    self.0.fetch_add(1, Ordering::Relaxed);
                }
                Ok(())
            }
        }
        let (base, requests) = wire::server(vec![wire::point_packet()]);
        let mut job = input(base, vec![segment(0, "Please add a group option.".into())]);
        let source = SourceSnapshot::new(&job.path, &job.segments).unwrap();
        let cancel = AtomicBool::new(false);
        extract_points(&source, &model(&job, &cancel))
            .await
            .unwrap();
        let request = requests
            .recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        let system = request["messages"][0]["content"].as_str().unwrap();
        let body: Value =
            serde_json::from_str(request["messages"][1]["content"].as_str().unwrap()).unwrap();
        let base_bytes = system.len() + body.to_string().len();
        let maximum = super::super::extraction::feedback_reservations()
            .into_iter()
            .map(|feedback| {
                let mut repaired = body.clone();
                repaired["repair"] = feedback;
                system.len() + repaired.to_string().len()
            })
            .max()
            .unwrap();
        let output = request["max_tokens"].as_u64().unwrap() as u32;
        job.settings.context_tokens = output + base_bytes.div_ceil(3) as u32;
        assert!(maximum > (job.settings.context_tokens - output) as usize * 3);
        job.settings.base_url = "http://127.0.0.1:1".into();
        let observed = Requests(AtomicUsize::new(0));
        let checked = EvidenceModel {
            observer: &observed,
            ..model(&job, &cancel)
        };
        let error = extract_points(&source, &checked)
            .await
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("context"), "{error}");
        assert_eq!(observed.0.load(Ordering::Relaxed), 0);

        let mut bad = wire::point_packet();
        bad["points"][0]["sources"] = json!(["S000099"]);
        let good = wire::point_packet();
        let (base, requests) = wire::server(vec![bad, good]);
        job.settings.base_url = base;
        job.settings.context_tokens = output + maximum.div_ceil(3) as u32;
        let points = extract_points(&source, &model(&job, &cancel))
            .await
            .unwrap();
        assert_eq!(points[0].point.text, "“Please add a group option.”");
        let requests: Vec<_> = requests.iter().collect();
        assert_eq!(requests.len(), 2);
        for request in requests {
            let bytes = request["messages"][0]["content"].as_str().unwrap().len()
                + request["messages"][1]["content"].as_str().unwrap().len();
            assert!(bytes <= (job.settings.context_tokens - output) as usize * 3);
        }
    }
}
