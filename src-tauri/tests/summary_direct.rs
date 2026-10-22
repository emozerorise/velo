//! Direct prose and citation/cache boundaries, without a semantic ledger.
use serde_json::json;
use std::sync::atomic::AtomicBool;
use velo_lib::{
    storage::settings_store::{SummaryFormat, SummarySettings},
    summary::{pipeline, JobInput, SummaryStore},
    transcript::TranscriptSegment,
};
#[allow(dead_code)]
#[path = "support/evidence_fixtures.rs"]
mod wire;

fn input(base: String, format: SummaryFormat) -> JobInput {
    JobInput {
        path: "/direct.wav".into(),
        segments: vec![TranscriptSegment {
            start: 37.38,
            end: 42.0,
            text: "ให้ดีฟ้าเป็นบุคคลก่อนนะ".into(),
        }],
        settings: SummarySettings {
            provider: "deepseek".into(),
            base_url: base,
            model: "synthetic".into(),
            format,
            language: "th".into(),
            ..Default::default()
        },
        vocabulary: "Default".into(),
        transcript_language: "th".into(),
        api_key: None,
    }
}
fn store(name: &str) -> SummaryStore {
    SummaryStore::open(
        std::env::temp_dir().join(format!("velo-direct-{}-{name}", std::process::id())),
    )
    .unwrap()
}
#[tokio::test]
async fn long_sources_keep_all_chunks_and_reduce_only_cited_notes() {
    let (base, requests) = wire::server(vec![
        json!("First [S000001]"),
        json!("First [S000001]"),
        json!("Second [S000002]"),
        json!("Second [S000002]"),
        json!("Third [S000003]"),
        json!("Third [S000003]"),
        json!("Combined [S000001] [S000003]"),
        json!("Combined [S000001] [S000003]"),
    ]);
    let mut job = input(base, SummaryFormat::Custom);
    job.settings.context_tokens = 8192;
    job.segments = (0..3)
        .map(|i| TranscriptSegment {
            start: i as f64 * 10.0,
            end: i as f64 * 10.0 + 9.0,
            text: "x".repeat(6000),
        })
        .collect();
    let result = pipeline::run(&store("long"), &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(result.markdown, "Combined [00:00] [00:20]");
    assert_eq!(requests.iter().count(), 8);
}
#[tokio::test]
async fn invalid_timeline_and_oversized_source_fail_without_network() {
    let mut job = input("http://127.0.0.1:1".into(), SummaryFormat::Custom);
    job.segments[0].start = f64::NAN;
    let s = store("invalid");
    let error = pipeline::run(&s, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap_err();
    assert!(error.to_string().contains("Invalid source timeline"));
    job.segments[0].start = 0.0;
    job.segments[0].text = "x".repeat(200_000);
    let error = pipeline::run(&s, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap_err();
    assert!(error.to_string().contains("context budget"));
    assert!(s.load(&job.path).is_none());
}
#[tokio::test]
async fn final_prose_review_uses_original_source_and_replaces_inferred_detail() {
    let (base, requests) = wire::server(vec![
        json!("Use the selected group, not the creator's personal rights [S000001]"),
        json!("Use the selected group [S000001]"),
    ]);
    let mut job = input(base, SummaryFormat::Custom);
    job.segments[0].text = "Use the selected group, not garbled-role itself".into();
    let result = pipeline::run(
        &store("prose-review"),
        &job,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(result.markdown, "Use the selected group [00:37]");
    let calls = requests.iter().collect::<Vec<_>>();
    assert_eq!(calls.len(), 2);
    assert!(calls[1]["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("not garbled-role itself"));
}
#[tokio::test]
async fn equivalent_numeric_source_ids_resolve_without_a_repair_call() {
    let (base, requests) = wire::server(vec![
        json!("Readable prose [S001]"),
        json!("Readable prose [S001]"),
    ]);
    let job = input(base, SummaryFormat::Custom);
    let result = pipeline::run(
        &store("short-id"),
        &job,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(result.markdown, "Readable prose [00:37]");
    assert_eq!(requests.iter().count(), 2);
}
#[tokio::test]
async fn repairs_missing_citation_once_using_the_same_source() {
    let (base, requests) = wire::server(vec![
        json!("A readable summary without a citation"),
        json!("A readable summary [S000001]"),
        json!("A readable summary [S000001]"),
    ]);
    let job = input(base, SummaryFormat::Custom);
    let result = pipeline::run(
        &store("repair"),
        &job,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(result.markdown, "A readable summary [00:37]");
    let calls = requests.iter().collect::<Vec<_>>();
    assert_eq!(calls.len(), 3);
    assert!(calls[1]["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("ให้ดีฟ้าเป็นบุคคลก่อนนะ"));
}
#[tokio::test]
async fn all_formats_review_rewritten_prose_once_and_resolve_source_time() {
    for (name, format) in [
        ("meeting", SummaryFormat::Meeting),
        ("key", SummaryFormat::KeyPoints),
        ("custom", SummaryFormat::Custom),
    ] {
        let (base, requests) = wire::server(vec![
            json!("กำหนดค่าเริ่มต้น (Default) เป็นบุคคล [S000001]"),
            json!("กำหนดค่าเริ่มต้น (Default) เป็นบุคคล [S000001]"),
        ]);
        let store = store(name);
        let job = input(base, format);
        let result = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
            .await
            .unwrap();
        assert_eq!(result.markdown, "กำหนดค่าเริ่มต้น (Default) เป็นบุคคล [00:37]");
        assert_eq!(result.omitted_uncertain_claims, None);
        assert_eq!(requests.iter().count(), 2);
        assert_eq!(job.segments[0].text, "ให้ดีฟ้าเป็นบุคคลก่อนนะ");
    }
}
#[tokio::test]
async fn invalid_citations_and_cancellation_preserve_previous_cache() {
    let store = store("preserve");
    let (base, _requests) = wire::server(vec![
        json!("Previous [S000001]"),
        json!("Previous [S000001]"),
    ]);
    let job = input(base, SummaryFormat::Custom);
    let old = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    for answer in [
        "Uncited prose",
        "Wrong [S000099]",
        "Invented time [00:38]",
        "",
    ] {
        let (base, _requests) = wire::server(vec![json!("Valid draft [S000001]"), json!(answer)]);
        assert!(pipeline::run(
            &store,
            &input(base, SummaryFormat::Custom),
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .is_err());
        assert_eq!(store.load(&job.path).unwrap().markdown, old.markdown);
    }
    assert!(
        pipeline::run(&store, &job, &AtomicBool::new(true), |_, _, _| {})
            .await
            .is_err()
    );
    assert_eq!(store.load(&job.path).unwrap().markdown, old.markdown);
}
