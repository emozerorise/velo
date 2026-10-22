//! Structural review/context failures preserve the previous cache.
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use velo_lib::{
    storage::settings_store::{SummaryFormat, SummarySettings},
    summary::{evidence_pipeline as pipeline, JobInput, Summary, SummaryStore},
    transcript::TranscriptSegment,
};
#[allow(dead_code)]
#[path = "support/evidence_fixtures.rs"]
mod wire;
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "velo-point-review-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn input(base_url: String) -> JobInput {
    JobInput {
        path: "/clip.wav".into(),
        segments: vec![TranscriptSegment {
            start: 1.25,
            end: 5.0,
            text: "Please add a group option.".into(),
        }],
        settings: SummarySettings {
            provider: "deepseek".into(),
            base_url,
            format: SummaryFormat::Custom,
            language: "en".into(),
            ..Default::default()
        },
        transcript_language: "en".into(),
        vocabulary: String::new(),
        api_key: None,
    }
}
fn store(dir: &Fixture) -> SummaryStore {
    let s = SummaryStore::open(dir.0.clone()).unwrap();
    s.save(&serde_json::from_value::<Summary>(json!({"path":"/clip.wav","model":"old","language":"en","markdown":"Previous summary","created_at":1,"source_segments":1})).unwrap()).unwrap();
    s
}
#[tokio::test]
async fn malformed_missing_fabricated_and_obsolete_reports_never_replace_cache() {
    let mut fabricated = wire::point_report();
    fabricated["checks"][0]["sources"] = json!(["S999999"]);
    let mut missing = wire::point_report();
    missing["checks"] = json!([]);
    for bad in [fabricated, missing, wire::report(), json!("{truncated")] {
        let (base, requests) = wire::server(vec![
            wire::point_packet(),
            wire::point_plan(),
            bad.clone(),
            bad,
        ]);
        let dir = Fixture::new();
        let store = store(&dir);
        assert!(
            pipeline::run(&store, &input(base), &AtomicBool::new(false), |_, _, _| {})
                .await
                .is_err()
        );
        assert_eq!(requests.iter().count(), 4);
        assert_eq!(
            store.load("/clip.wav").unwrap().markdown,
            "Previous summary"
        );
    }
}
#[tokio::test]
async fn unknown_point_reference_gets_one_structural_repair_not_replacement_prose() {
    let mut bad = wire::point_plan();
    bad["blocks"][1]["claims"] = json!(["P999999"]);
    let (base, requests) = wire::server(vec![
        wire::point_packet(),
        bad,
        wire::point_plan(),
        wire::point_report(),
    ]);
    let dir = Fixture::new();
    let store = store(&dir);
    assert_eq!(
        pipeline::run(&store, &input(base), &AtomicBool::new(false), |_, _, _| {})
            .await
            .unwrap()
            .markdown,
        "## Overview\n\n“Please add a group option.” [00:01]"
    );
    assert_eq!(requests.iter().count(), 4);
}
#[tokio::test]
async fn indivisible_oversized_source_fails_before_http_without_truncation() {
    let mut job = input("http://127.0.0.1:1".into());
    job.segments[0].text = "ก".repeat(1000);
    let dir = Fixture::new();
    let store = store(&dir);
    assert!(
        pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
            .await
            .unwrap_err()
            .to_string()
            .contains("context")
    );
    assert_eq!(
        store.load("/clip.wav").unwrap().markdown,
        "Previous summary"
    );
}
#[tokio::test]
async fn exhausted_followup_and_schema_slots_never_make_a_third_review_call() {
    let mut follow = wire::point_report();
    follow["need_sources"] = json!(["S000001"]);
    for suffix in [
        vec![follow.clone(), json!("bad schema")],
        vec![json!("bad schema"), follow.clone()],
        vec![follow.clone(), follow.clone()],
    ] {
        let mut answers = vec![wire::point_packet(), wire::point_plan()];
        answers.extend(suffix);
        let (base, requests) = wire::server(answers);
        let dir = Fixture::new();
        let store = store(&dir);
        assert!(
            pipeline::run(&store, &input(base), &AtomicBool::new(false), |_, _, _| {})
                .await
                .is_err()
        );
        assert_eq!(requests.iter().count(), 4);
        assert_eq!(
            store.load("/clip.wav").unwrap().markdown,
            "Previous summary"
        );
    }
    let mut unknown = follow;
    unknown["need_sources"] = json!(["S999999"]);
    let (base, requests) = wire::server(vec![wire::point_packet(), wire::point_plan(), unknown]);
    let dir = Fixture::new();
    let store = store(&dir);
    assert!(
        pipeline::run(&store, &input(base), &AtomicBool::new(false), |_, _, _| {})
            .await
            .is_err()
    );
    assert_eq!(requests.iter().count(), 3);
}
#[tokio::test]
async fn small_review_receives_full_raw_snapshot_including_later_clarification() {
    let (base, requests) = wire::server(vec![
        wire::point_packet(),
        wire::point_plan(),
        wire::point_report(),
    ]);
    let mut job = input(base);
    job.segments.push(TranscriptSegment {
        start: 150.25,
        end: 153.0,
        text: "Later clarification: this remains a request, not an implemented feature.".into(),
    });
    let dir = Fixture::new();
    let store = store(&dir);
    pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    let requests: Vec<_> = requests.iter().collect();
    let review: Value =
        serde_json::from_str(requests[2]["messages"][1]["content"].as_str().unwrap()).unwrap();
    assert_eq!(
        review["sources"][1]["text"],
        "Later clarification: this remains a request, not an implemented feature."
    );
}

#[tokio::test]
async fn meeting_outcome_headings_cannot_be_certified_by_provider_verdicts() {
    for heading in ["decisions", "actions", "open_questions"] {
        for exhausted in [false, true] {
            let bad = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"heading","heading":heading},{"kind":"list","claims":["P000001"]}]});
            let neutral = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"heading","heading":"topics"},{"kind":"list","claims":["P000001"]}]});
            let (base, requests) = wire::server(vec![
                wire::point_packet(),
                bad.clone(),
                if exhausted { bad } else { neutral },
                wire::point_report(),
            ]);
            let mut job = input(base);
            job.settings.format = SummaryFormat::Meeting;
            job.settings.instructions = "Use decisions, action items and open questions.".into();
            job.segments[0].text = "Maybe leave it for later?".into();
            let dir = Fixture::new();
            let store = store(&dir);
            let result = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {}).await;
            if !exhausted {
                assert_eq!(
                    result.unwrap().markdown,
                    "## Topics discussed\n\n- “Maybe leave it for later?” [00:01]"
                );
            } else {
                assert!(result.is_err());
                assert_eq!(
                    store.load("/clip.wav").unwrap().markdown,
                    "Previous summary"
                );
            }
            let requests: Vec<_> = requests.iter().collect();
            assert_eq!(requests.len(), if exhausted { 3 } else { 4 });
            let repaired: Value =
                serde_json::from_str(requests[2]["messages"][1]["content"].as_str().unwrap())
                    .unwrap();
            assert!(repaired["repair"].as_str().unwrap().contains("heading"));
            if !exhausted {
                let reviewed: Value =
                    serde_json::from_str(requests[3]["messages"][1]["content"].as_str().unwrap())
                        .unwrap();
                assert_eq!(reviewed["occurrences"][0]["heading"], "topics");
                assert_eq!(
                    reviewed["occurrences"][0]["text"],
                    "“Maybe leave it for later?”"
                );
            }
        }
    }
}

#[tokio::test]
async fn reviewer_source_ids_resolve_raw_evidence_without_copying_quotations() {
    let report = json!({"job":"$JOB","checks":[{"occurrence":"O000001","verdict":"supported","sources":["S000001"]}],"need_sources":[]});
    let (base, requests) = wire::server(vec![wire::point_packet(), wire::point_plan(), report]);
    let dir = Fixture::new();
    let store = store(&dir);
    let mut job = input(base);
    job.segments[0].text =
        "Repeated repeated. May it disappear only when inspection finishes?".into();
    let summary = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(summary.markdown,"## Overview\n\n“Repeated repeated. May it disappear only when inspection finishes?” [00:01]");
    assert_eq!(requests.iter().count(), 3);
}
