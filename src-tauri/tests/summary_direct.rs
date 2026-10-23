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
async fn indivisible_rows_are_not_lossily_condensed_to_force_a_small_context_fit() {
    let mut job = input("http://127.0.0.1:1".into(), SummaryFormat::Custom);
    job.settings.context_tokens = 8192;
    job.segments = (0..3)
        .map(|i| TranscriptSegment {
            start: i as f64 * 10.0,
            end: i as f64 * 10.0 + 9.0,
            text: "x".repeat(30000),
        })
        .collect();
    let cache = store("long-indivisible");
    let result = pipeline::run(&cache, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap_err();
    assert_eq!(result.to_string(), "summary_context_limit");
    assert!(cache.load(&job.path).is_none());
    assert_eq!(job.segments.len(), 3);
    assert!(job.segments.iter().all(|row| row.text.len() == 30000));
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
    assert_eq!(error.to_string(), "summary_context_limit");
    assert!(s.load(&job.path).is_none());
}
#[tokio::test]
async fn final_prose_is_returned_without_an_ai_review() {
    let (base, requests) = wire::server(vec![json!("Use the selected group [S000001]")]);
    let job = input(base, SummaryFormat::Custom);
    let result = pipeline::run(&store("prose"), &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    assert_eq!(result.markdown, "Use the selected group [00:37]");
    assert_eq!(requests.iter().count(), 1);
}
#[tokio::test]
async fn equivalent_numeric_source_ids_resolve_without_a_repair_call() {
    let (base, requests) = wire::server(vec![json!("Readable prose [S001]")]);
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
    assert_eq!(requests.iter().count(), 1);
}

#[tokio::test]
async fn supplied_source_ranges_resolve_each_original_clock_without_a_repair_call() {
    let (base, requests) = wire::server(vec![json!("Readable prose [S000001-S000003]")]);
    let mut job = input(base, SummaryFormat::Custom);
    job.segments = vec![
        TranscriptSegment {
            start: 37.38,
            end: 42.0,
            text: "First".into(),
        },
        TranscriptSegment {
            start: 43.76,
            end: 45.0,
            text: "Second".into(),
        },
        TranscriptSegment {
            start: 3602.9,
            end: 3608.0,
            text: "Third".into(),
        },
    ];
    let result = pipeline::run(
        &store("range-id"),
        &job,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert_eq!(result.markdown, "Readable prose [00:37] [00:43] [1:00:02]");
    assert_eq!(requests.iter().count(), 1);
}
#[tokio::test]
async fn all_formats_write_prose_once_and_resolve_source_time() {
    for (name, format) in [
        ("meeting", SummaryFormat::Meeting),
        ("key", SummaryFormat::KeyPoints),
        ("custom", SummaryFormat::Custom),
    ] {
        let (base, requests) =
            wire::server(vec![json!("กำหนดค่าเริ่มต้น (Default) เป็นบุคคล [S000001]")]);
        let store = store(name);
        let job = input(base, format);
        let result = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
            .await
            .unwrap();
        assert_eq!(result.markdown, "กำหนดค่าเริ่มต้น (Default) เป็นบุคคล [00:37]");
        assert_eq!(result.omitted_uncertain_claims, None);
        assert_eq!(requests.iter().count(), 1);
        assert_eq!(job.segments[0].text, "ให้ดีฟ้าเป็นบุคคลก่อนนะ");
    }
}
#[tokio::test]
async fn invalid_citations_and_cancellation_preserve_previous_cache() {
    let store = store("preserve");
    let (base, _requests) = wire::server(vec![json!("Previous [S000001]")]);
    let job = input(base, SummaryFormat::Custom);
    let old = pipeline::run(&store, &job, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    for answer in [
        "Uncited prose",
        "Wrong [S000099]",
        "Wrong [S000001-S000099] [S000001]",
        "Invented time [00:38]",
        "",
    ] {
        let (base, _requests) = wire::server(vec![json!(answer)]);
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
