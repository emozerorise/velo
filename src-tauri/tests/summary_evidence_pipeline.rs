//! Archived extractive pipeline/cache boundaries; not the app's current runner.
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use velo_lib::{
    storage::settings_store::{SummaryFormat, SummarySettings},
    summary::{evidence_pipeline as pipeline, JobInput, Summary, SummaryStore},
    transcript::TranscriptSegment,
};
#[cfg(feature = "summary-evidence-qa")]
#[path = "support/evidence_acceptance.rs"]
mod evidence_acceptance;
#[allow(dead_code)]
#[path = "support/evidence_fixtures.rs"]
mod wire;

#[cfg(feature = "summary-evidence-qa")]
#[tokio::test]
async fn observer_preserves_requests_text_and_private_credential_boundaries() {
    let dir = Fixture::new();
    let artifacts = evidence_acceptance::Artifacts::new(dir.0.join("artifacts"));
    let mut documents = Vec::new();
    let mut calls = Vec::new();
    for observed in [false, true] {
        let report = wire::point_report();
        let (base, requests) = wire::server(vec![
            wire::protected_point_packet(),
            wire::point_plan(),
            report,
        ]);
        let store =
            SummaryStore::open(dir.0.join(if observed { "observed" } else { "default" })).unwrap();
        let mut job = input(base, SummaryFormat::Custom);
        job.segments[0].text = "Only when inspection finishes can the item be removed.".into();
        job.api_key = Some("synthetic-secret-sentinel".into());
        let cancel = AtomicBool::new(false);
        let result = if observed {
            pipeline::run_observed(&store, &job, &cancel, |_, _, _| {}, &artifacts).await
        } else {
            pipeline::run(&store, &job, &cancel, |_, _, _| {}).await
        }
        .unwrap();
        documents.push(result.markdown);
        let requests: Vec<_> = requests
            .iter()
            .map(|r| {
                let token = payload(&r)["job"].as_str().unwrap().to_owned();
                r.to_string().replace(&token, "SAME-JOB")
            })
            .collect();
        assert_eq!(requests.len(), 3);
        calls.push(requests);
    }
    assert_eq!(documents[0], documents[1]);
    assert_eq!(calls[0], calls[1]);
    let manifest = artifacts.manifest();
    assert_eq!(manifest["extraction_packets"], 1);
    assert_eq!(manifest["verification_batches"], 1);
    assert_eq!(manifest["requests"], 3);
    assert_eq!(manifest["ceiling"], 6);
    for file in std::fs::read_dir(dir.0.join("artifacts")).unwrap() {
        let bytes = std::fs::read(file.unwrap().path()).unwrap();
        let text = String::from_utf8(bytes).unwrap();
        assert!(!text.contains("synthetic-secret-sentinel"));
        let value: Value = serde_json::from_str(&text).unwrap();
        let payload = value["payload"].as_object().unwrap();
        if value["event"] == "points" {
            assert_eq!(
                value["payload"]["points"][0]["point"]["text"],
                "“Only when inspection finishes can the item be removed.”"
            );
            assert!(value["payload"]["points"][0]["point"]
                .get("parts")
                .is_none());
        }
        if value["event"] == "response" && value["payload"]["stage"] == "extracting" {
            let packet: Value =
                serde_json::from_str(value["payload"]["text"].as_str().unwrap()).unwrap();
            assert_eq!(packet["points"][0]["sources"][0], "S000001");
        }
        for key in [
            "api_key",
            "headers",
            "authorization",
            "base_url",
            "JobInput",
        ] {
            assert!(!payload.contains_key(key));
        }
    }
}
#[cfg(feature = "summary-evidence-qa")]
#[test]
fn acceptance_counts_source_and_occurrence_id_sets_not_followup_context() {
    use pipeline::EvidenceObserver;
    let dir = Fixture::new();
    let artifacts = evidence_acceptance::Artifacts::new(dir.0.join("counted"));
    for rows in [
        json!([{"source":"S000001","text":"first"},{"source":"S000002","text":"second"}]),
        json!([{"source":"S000002","text":"second"},{"source":"S000001","text":"first"}]),
    ] {
        artifacts.record("request",&json!({"stage":"extracting","system":"synthetic","user":json!({"sources":rows}).to_string(),"max_tokens":4096,"context_tokens":32768})).unwrap();
    }
    for (id, source) in [
        ("O000001", "S000001"),
        ("O000001", "S000030"),
        ("O000002", "S000002"),
    ] {
        artifacts.record("request",&json!({"stage":"checking","system":"synthetic","user":json!({"occurrences":[{"id":id}],"sources":[{"source":source}]}).to_string(),"max_tokens":4096,"context_tokens":32768})).unwrap();
    }
    let manifest = artifacts.manifest();
    assert_eq!(manifest["extraction_packets"], 1);
    assert_eq!(manifest["verification_batches"], 2);
    assert_eq!(manifest["requests"], 5);
    assert_eq!(manifest["ceiling"], 8);
}
#[cfg(feature = "summary-evidence-qa")]
#[test]
fn acceptance_artifacts_reject_nonallowlisted_metadata_but_keep_source_wording() {
    use pipeline::EvidenceObserver;
    let dir = Fixture::new();
    let artifacts = evidence_acceptance::Artifacts::new(dir.0.join("private"));
    assert!(artifacts
        .record(
            "request",
            &json!({"stage":"checking","api_key":"synthetic-secret-sentinel"})
        )
        .is_err());
    assert!(artifacts
        .record("JobInput", &json!({"api_key":"synthetic-secret-sentinel"}))
        .is_err());
    assert_eq!(std::fs::read_dir(dir.0.join("private")).unwrap().count(), 0);
    artifacts
        .record(
            "composed",
            &json!({"markdown":"The speaker discussed an API key and headers."}),
        )
        .unwrap();
    assert_eq!(std::fs::read_dir(dir.0.join("private")).unwrap().count(), 1);
}
#[cfg(feature = "summary-evidence-qa")]
#[test]
fn acceptance_distinguishes_escaped_source_markup_from_application_references() {
    assert_eq!(
        evidence_acceptance::application_labels("Source \\[00:10\\] \\[S000099\\] [00:12]"),
        vec!["00:12"]
    );
    assert!(evidence_acceptance::is_plain_paragraph(
        "Literal \\# title \\- item \\[00:10\\] [00:12]"
    ));
    assert!(evidence_acceptance::is_plain_paragraph(
        "\\# heading in source [00:12]"
    ));
    for invalid in [
        "## Title\n\nParagraph [00:12]",
        "- Bullet [00:12]",
        "First [00:12]\n\nSecond [00:12]",
        "",
    ] {
        assert!(!evidence_acceptance::is_plain_paragraph(invalid));
    }
}

struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "velo-points-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn input(base_url: String, format: SummaryFormat) -> JobInput {
    JobInput {
        path: "/synthetic.wav".into(),
        segments: vec![TranscriptSegment {
            start: 1.25,
            end: 5.0,
            text: "Please add a group option.".into(),
        }],
        settings: SummarySettings {
            provider: "deepseek".into(),
            base_url,
            language: "auto".into(),
            format,
            instructions: "PRIVATE CUSTOM FOCUS".into(),
            ..Default::default()
        },
        vocabulary: "Default".into(),
        transcript_language: "en".into(),
        api_key: None,
    }
}
fn previous(store: &SummaryStore) {
    store.save(&serde_json::from_value::<Summary>(json!({"path":"/synthetic.wav","model":"old","language":"en","markdown":"Previous summary","created_at":1,"source_segments":1})).unwrap()).unwrap();
}
fn payload(req: &Value) -> Value {
    serde_json::from_str(req["messages"][1]["content"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn extraction_repairs_unknown_source_once_without_inventing_wording() {
    let mut bad = wire::point_packet();
    bad["points"][0]["sources"] = json!(["S000099"]);
    let corrected = wire::point_packet();
    let plan = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"paragraph","claims":["P000001"]}]});
    let (base, requests) = wire::server(vec![bad, corrected, plan, wire::point_report()]);
    let dir = Fixture::new();
    let store = SummaryStore::open(dir.0.clone()).unwrap();
    let summary = pipeline::run(
        &store,
        &input(base, SummaryFormat::Custom),
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(summary.markdown, "“Please add a group option.” [00:01]");
    let requests: Vec<_> = requests.iter().collect();
    assert_eq!(requests.len(), 4);
    let repair = payload(&requests[1])["repair"].clone();
    assert_eq!(repair["category"], "invalid_source");
    assert_eq!(repair["point"], 0);
    assert_eq!(repair["source"], 0);
    assert!(repair.get("quote").is_none());
    assert!(repair["instruction"]
        .as_str()
        .unwrap()
        .contains("source IDs"));
}
#[tokio::test]
async fn whole_source_excerpts_reach_planner_reviewer_and_cache() {
    let packet = wire::protected_point_packet();
    let report = wire::point_report();
    let plan = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"paragraph","claims":["P000001"]}]});
    let (base, requests) = wire::server(vec![packet, plan, report]);
    let mut job = input(base, SummaryFormat::Custom);
    job.segments[0] = TranscriptSegment {
        start: 12.75,
        end: 16.25,
        text: "Only when inspection finishes can the item be removed.".into(),
    };
    let dir = Fixture::new();
    let store = SummaryStore::open(dir.0.clone()).unwrap();
    let summary = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(
        summary.markdown,
        "“Only when inspection finishes can the item be removed.” [00:12]"
    );
    assert_eq!(
        store.load("/synthetic.wav").unwrap().markdown,
        summary.markdown
    );
    let requests: Vec<_> = requests.iter().collect();
    assert_eq!(requests.len(), 3);
    let system = requests[0]["messages"][0]["content"].as_str().unwrap();
    let schema = system
        .lines()
        .find_map(|line| line.strip_prefix("Schema: "))
        .unwrap()
        .trim_end_matches('.');
    let schema: Value = serde_json::from_str(schema).unwrap();
    assert_eq!(schema["job"], payload(&requests[0])["job"]);
    assert!(schema["points"][0]["sources"].is_array());
    assert!(schema["points"][0].get("text").is_none());
    let planned = payload(&requests[1]);
    let reviewed = payload(&requests[2]);
    assert_eq!(
        planned["points"][0]["text"],
        "“Only when inspection finishes can the item be removed.”"
    );
    assert_eq!(planned["points"][0].as_object().unwrap().len(), 3);
    assert!(planned["points"][0]["words"].as_u64().unwrap() > 1);
    assert_eq!(planned["format"], "custom");
    assert!(planned["points"][0]["point"].get("parts").is_none());
    assert_eq!(
        reviewed["occurrences"][0]["text"],
        "“Only when inspection finishes can the item be removed.”"
    );
    assert_eq!(
        reviewed["occurrences"][0]["required"][0]["quote"],
        "Only when inspection finishes can the item be removed."
    );
    assert_eq!(reviewed["sources"][0]["text"], job.segments[0].text);
    assert_eq!(job.segments[0].start, 12.75);
}

#[tokio::test]
async fn explicit_custom_word_limit_repairs_whole_selection_without_truncating_quotes() {
    for exhausted in [false, true] {
        let packet =
            json!({"job":"$JOB","points":[{"sources":["S000001"]},{"sources":["S000002"]}]});
        let long = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"paragraph","claims":["P000001"]}]});
        let short = json!({"job":"$JOB","status":"supported","blocks":[{"kind":"paragraph","claims":["P000002"]}]});
        let mut answers = vec![packet, long.clone(), if exhausted { long } else { short }];
        if !exhausted {
            answers.push(json!({"job":"$JOB","checks":[{"occurrence":"O000001","verdict":"supported","sources":["S000002"]}],"need_sources":[]}));
        }
        let (base, requests) = wire::server(answers);
        let dir = Fixture::new();
        let store = SummaryStore::open(dir.0.clone()).unwrap();
        previous(&store);
        let mut job = input(base, SummaryFormat::Custom);
        job.settings.instructions = "One paragraph, at most 2 words.".into();
        job.segments[0].text = "Only after completion may it disappear?".into();
        job.segments.push(TranscriptSegment {
            start: 6.25,
            end: 9.0,
            text: "Keep records.".into(),
        });
        let result = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {}).await;
        if exhausted {
            assert!(result.is_err());
            assert_eq!(store.load(&job.path).unwrap().markdown, "Previous summary");
        } else {
            assert_eq!(result.unwrap().markdown, "“Keep records.” [00:06]");
        }
        let requests: Vec<_> = requests.iter().collect();
        assert_eq!(requests.len(), if exhausted { 3 } else { 4 });
        assert_eq!(payload(&requests[1])["max_words"], 2);
        assert_eq!(payload(&requests[1])["points"][0]["words"], 6);
        assert_eq!(
            payload(&requests[2])["repair"]["category"],
            "word_limit_exceeded"
        );
        assert_eq!(payload(&requests[2])["repair"]["limit"], 2);
    }
}
#[tokio::test]
async fn oversized_extraction_is_repaired_once_not_truncated_or_published() {
    for exhausted in [false, true] {
        let mut packet = wire::point_packet();
        packet["points"] = json!(vec![packet["points"][0].clone(); 7]);
        let answers = if exhausted {
            vec![packet.clone(), packet]
        } else {
            vec![
                packet,
                wire::point_packet(),
                wire::point_plan(),
                wire::point_report(),
            ]
        };
        let (base, requests) = wire::server(answers);
        let dir = Fixture::new();
        let store = SummaryStore::open(dir.0.clone()).unwrap();
        previous(&store);
        let cache = std::fs::read_dir(&dir.0)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let before = std::fs::read(&cache).unwrap();
        let result = pipeline::run(
            &store,
            &input(base, SummaryFormat::Custom),
            &AtomicBool::new(false),
            |_, _, _| {},
        )
        .await;
        if exhausted {
            assert!(result.is_err());
            assert_eq!(std::fs::read(&cache).unwrap(), before);
        } else {
            assert!(result.is_ok());
        }
        let requests: Vec<_> = requests.iter().collect();
        assert_eq!(requests.len(), if exhausted { 2 } else { 4 });
        let repair = payload(&requests[1])["repair"].clone();
        assert_eq!(repair["category"], "point_limit_exceeded");
        assert_eq!(repair["actual"], 7);
        assert_eq!(repair["limit"], 1);
    }
}
#[tokio::test]
async fn all_nine_provider_formats_extract_plan_review_whole_points_and_save_once() {
    for provider in ["ollama", "openai", "deepseek"] {
        for format in [
            SummaryFormat::Meeting,
            SummaryFormat::KeyPoints,
            SummaryFormat::Custom,
        ] {
            let (base, requests) = wire::server(vec![
                wire::point_packet(),
                wire::point_plan(),
                wire::point_report(),
            ]);
            let dir = Fixture::new();
            let store = SummaryStore::open(dir.0.clone()).unwrap();
            let mut job = input(base, format);
            job.settings.provider = provider.into();
            let stages = std::sync::Mutex::new(Vec::new());
            let summary = pipeline::run(&store, &job, &AtomicBool::new(false), |stage, _, _| {
                stages.lock().unwrap().push(stage.to_owned())
            })
            .await
            .unwrap();
            assert_eq!(
                summary.markdown,
                "## Overview\n\n“Please add a group option.” [00:01]"
            );
            assert_eq!(summary.language, "en");
            assert_eq!(summary.format, format);
            assert_eq!(
                store.load("/synthetic.wav").unwrap().markdown,
                summary.markdown
            );
            assert_eq!(
                *stages.lock().unwrap(),
                [
                    "extracting_evidence",
                    "checking_evidence",
                    "composing_summary"
                ]
            );
            let bodies: Vec<_> = requests.iter().collect();
            assert_eq!(bodies.len(), 3);
            assert!(!bodies[0].to_string().contains("PRIVATE CUSTOM FOCUS"));
            assert!(bodies[1].to_string().contains("PRIVATE CUSTOM FOCUS"));
            assert!(!bodies[2].to_string().contains("PRIVATE CUSTOM FOCUS"));
            assert_eq!(
                payload(&bodies[0])["sources"][0]["text"],
                "Please add a group option."
            );
            assert_eq!(payload(&bodies[2])["occurrences"][0]["heading"], "overview");
            assert!(payload(&bodies[2])["sources"]
                .to_string()
                .contains("Please add a group option."));
            assert!(payload(&bodies[2])
                .to_string()
                .contains("“Please add a group option.”"));
            for body in bodies {
                if provider == "ollama" {
                    assert_eq!(body["think"], false);
                    assert!(body["options"]["num_predict"].as_u64().unwrap() > 0);
                } else {
                    assert!(body["max_tokens"].as_u64().unwrap() > 0);
                    if provider == "deepseek" {
                        assert_eq!(body["thinking"]["type"], "disabled");
                    } else {
                        assert!(body.get("thinking").is_none());
                    }
                }
            }
        }
    }
}
#[tokio::test]
async fn cancellation_inside_each_http_await_and_before_save_preserves_cache_bytes() {
    for stop in 1..=3 {
        let answers = vec![
            wire::point_packet(),
            wire::point_plan(),
            wire::point_report(),
        ]
        .into_iter()
        .take(stop)
        .collect();
        let (base, requests, reached, release) = wire::server_gated(answers, stop);
        let dir = Fixture::new();
        let store = SummaryStore::open(dir.0.clone()).unwrap();
        previous(&store);
        let before = std::fs::read(
            std::fs::read_dir(&dir.0)
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path(),
        )
        .unwrap();
        let cancel = AtomicBool::new(false);
        let job = input(base, SummaryFormat::Custom);
        let result = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let (result, ()) =
                tokio::join!(pipeline::run(&store, &job, &cancel, |_, _, _| {}), async {
                    reached.await.unwrap();
                    cancel.store(true, Ordering::Relaxed);
                });
            result
        })
        .await
        .unwrap();
        release.send(()).unwrap();
        let error = result.unwrap_err();
        assert!(
            error.to_string().to_lowercase().contains("cancel"),
            "await {stop}: {error:?}"
        );
        assert_eq!(requests.iter().count(), stop);
        assert_eq!(
            std::fs::read(
                std::fs::read_dir(&dir.0)
                    .unwrap()
                    .next()
                    .unwrap()
                    .unwrap()
                    .path()
            )
            .unwrap(),
            before
        );
    }
    let (base, requests) = wire::server(vec![
        wire::point_packet(),
        wire::point_plan(),
        wire::point_report(),
    ]);
    let dir = Fixture::new();
    let store = SummaryStore::open(dir.0.clone()).unwrap();
    previous(&store);
    let cancel = AtomicBool::new(false);
    let result = pipeline::run(
        &store,
        &input(base, SummaryFormat::Custom),
        &cancel,
        |s, _, _| {
            if s == "composing_summary" {
                cancel.store(true, Ordering::Relaxed);
            }
        },
    )
    .await;
    assert!(result.unwrap_err().to_string().contains("cancel"));
    assert_eq!(requests.iter().count(), 3);
    assert_eq!(
        store.load("/synthetic.wav").unwrap().markdown,
        "Previous summary"
    );
}
#[tokio::test]
async fn structural_repairs_are_bounded_and_current_job_tokens_are_required() {
    for stage in 0..3 {
        let good = [
            wire::point_packet(),
            wire::point_plan(),
            wire::point_report(),
        ];
        let mut answers = good[..stage].to_vec();
        answers.extend([json!("malformed"), json!("still malformed")]);
        let (base, requests) = wire::server(answers);
        let dir = Fixture::new();
        let store = SummaryStore::open(dir.0.clone()).unwrap();
        previous(&store);
        assert!(pipeline::run(
            &store,
            &input(base, SummaryFormat::Custom),
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .is_err());
        assert_eq!(requests.iter().count(), stage + 2);
        assert_eq!(
            store.load("/synthetic.wav").unwrap().markdown,
            "Previous summary"
        );
    }
    let mut wrong = wire::point_packet();
    wrong["job"] = json!("OTHER-JOB");
    let (base, requests) = wire::server(vec![
        wrong,
        wire::point_packet(),
        wire::point_plan(),
        wire::point_report(),
    ]);
    let dir = Fixture::new();
    let store = SummaryStore::open(dir.0.clone()).unwrap();
    pipeline::run(
        &store,
        &input(base, SummaryFormat::Custom),
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    let requests: Vec<_> = requests.iter().collect();
    assert_eq!(requests.len(), 4);
    assert_eq!(payload(&requests[1])["repair"]["category"], "job_mismatch");
}
#[tokio::test]
async fn selected_rejections_count_once_and_all_rejected_or_unsupported_preserves_old_cache() {
    for all in [false, true] {
        let mut packet = wire::point_packet();
        let mut other = packet["points"][0].clone();
        other["sources"] = json!(["S000002"]);
        packet["points"].as_array_mut().unwrap().push(other);
        let mut plan = wire::point_plan();
        plan["blocks"][1]["claims"] = json!(["P000001", "P000002"]);
        let mut report = wire::point_report();
        let mut rejected = report["checks"][0].clone();
        rejected["occurrence"] = json!("O000002");
        rejected["verdict"] = json!("unsupported");
        rejected["sources"] = json!([]);
        report["checks"].as_array_mut().unwrap().push(rejected);
        if all {
            report["checks"][0]["verdict"] = json!("unclear");
        }
        let (base, requests) = wire::server(vec![packet, plan, report]);
        let dir = Fixture::new();
        let store = SummaryStore::open(dir.0.clone()).unwrap();
        previous(&store);
        let mut job = input(base, SummaryFormat::Custom);
        job.segments.push(TranscriptSegment {
            start: 5.5,
            end: 9.0,
            text: "Could the permission scope change?".into(),
        });
        let result = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {}).await;
        if all {
            assert!(result.is_err());
            assert_eq!(
                store.load("/synthetic.wav").unwrap().markdown,
                "Previous summary"
            );
        } else {
            let summary = result.unwrap();
            assert_eq!(summary.omitted_uncertain_claims, Some(1));
            assert!(!summary.markdown.contains("permission"));
        }
        assert_eq!(requests.iter().count(), 3);
    }
    let (base, requests) = wire::server(vec![
        wire::point_packet(),
        json!({"job":"$JOB","status":"unsupported","blocks":[]}),
    ]);
    let dir = Fixture::new();
    let store = SummaryStore::open(dir.0.clone()).unwrap();
    previous(&store);
    assert_eq!(
        pipeline::run(
            &store,
            &input(base, SummaryFormat::Custom),
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .unwrap_err()
        .to_string(),
        "unsupported_summary_layout"
    );
    assert_eq!(requests.iter().count(), 2);
    assert_eq!(
        store.load("/synthetic.wav").unwrap().markdown,
        "Previous summary"
    );
}
