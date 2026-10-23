//! Public pipeline regressions: cloud calls are replaced only at HTTP transport.
#[allow(dead_code)]
#[path = "support/summary_model.rs"]
mod wire;
use std::sync::atomic::{AtomicBool, Ordering};
use velo_lib::{
    storage::settings_store::{SummaryFormat, SummarySettings},
    summary::{pipeline, JobInput, SummaryStore},
    transcript::TranscriptSegment,
};
use wire::{Model, Reply};
fn job(base: String, count: usize, bytes: usize) -> JobInput {
    JobInput {
        path: "/simple.wav".into(),
        segments: (0..count)
            .map(|i| TranscriptSegment {
                start: i as f64 * 10.,
                end: i as f64 * 10. + 9.,
                text: format!(
                    "ROW{i}: condition only after completion. {}",
                    "x".repeat(bytes)
                ),
            })
            .collect(),
        settings: SummarySettings {
            provider: "deepseek".into(),
            base_url: base,
            language: "en".into(),
            format: SummaryFormat::Custom,
            instructions: "One paragraph".into(),
            ..Default::default()
        },
        vocabulary: String::new(),
        transcript_language: "en".into(),
        api_key: None,
    }
}
fn store(name: &str) -> SummaryStore {
    SummaryStore::open(
        std::env::temp_dir().join(format!("velo-simple-{}-{name}", std::process::id())),
    )
    .unwrap()
}
fn reply(req: &serde_json::Value) -> Reply {
    let text = req["messages"][1]["content"].as_str().unwrap();
    let Some(rest) = text.split("[S").nth(1) else {
        return Reply::complete(r#"{"kind":"single_paragraph"}"#);
    };
    let id = rest.split(']').next().unwrap();
    Reply::complete(format!("Supported condition [S{id}]."))
}
#[tokio::test]
async fn fitting_source_over_24kb_is_sent_once_in_every_format() {
    for format in [
        SummaryFormat::Custom,
        SummaryFormat::Meeting,
        SummaryFormat::KeyPoints,
    ] {
        let model = Model::start(|r, _| reply(r));
        let mut input = job(model.base.clone(), 32, 1000);
        input.settings.format = format;
        let result = pipeline::run(
            &store(&format!("direct-{format:?}")),
            &input,
            &AtomicBool::new(false),
            |_, _, _| {},
        )
        .await
        .unwrap();
        assert!(result.markdown.contains("[00:00]"));
        let calls = model.requests.lock().unwrap();
        assert_eq!(calls.len(), 1, "no review, repair or fixed-size partition");
        let source = calls[0]["messages"][1]["content"].as_str().unwrap();
        for i in 0..32 {
            assert!(source.contains(&format!("ROW{i}:")));
        }
    }
}
#[tokio::test]
async fn invalid_response_stops_after_one_call_and_preserves_cache() {
    let cache = store("invalid");
    let good = Model::start(|r, _| reply(r));
    let old = pipeline::run(
        &cache,
        &job(good.base.clone(), 1, 10),
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    for answer in ["No citation", "Wrong [S999999]", "", "Invented [00:38]"] {
        let bad = Model::start(move |_, _| Reply::complete(answer));
        assert!(pipeline::run(
            &cache,
            &job(bad.base.clone(), 1, 10),
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .is_err());
        assert_eq!(bad.requests.lock().unwrap().len(), 1);
        assert_eq!(cache.load("/simple.wav").unwrap().markdown, old.markdown);
    }
}
#[tokio::test]
async fn oversized_source_covers_every_row_then_combines_once() {
    let model = Model::start(|r, _| reply(r));
    let input = job(model.base.clone(), 120, 1500);
    let mut stages = Vec::new();
    let stages = std::sync::Mutex::new(&mut stages);
    let result = pipeline::run(
        &store("long"),
        &input,
        &AtomicBool::new(false),
        |s, _, _| stages.lock().unwrap().push(s.to_owned()),
    )
    .await
    .unwrap();
    assert!(!result.markdown.is_empty());
    let calls = model.requests.lock().unwrap();
    assert!(calls.len() > 2 && calls.len() < 10);
    for i in 0..120 {
        assert!(calls[..calls.len() - 1]
            .iter()
            .any(|r| r["messages"][1]["content"]
                .as_str()
                .unwrap()
                .contains(&format!("ROW{i}:"))));
    }
    assert!(!calls.last().unwrap()["messages"][1]["content"]
        .as_str()
        .unwrap()
        .contains("ROW0:"));
    assert_eq!(
        stages
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.as_str() == "assembling_summary")
            .count(),
        1
    );
    assert!(!stages
        .lock()
        .unwrap()
        .iter()
        .any(|s| s.contains("consistency") || s.contains("replacing")));
}
#[tokio::test]
async fn output_limit_splits_source_without_retrying_the_same_request() {
    let model = Model::start(|r, i| if i == 0 { Reply::limited() } else { reply(r) });
    let input = job(model.base.clone(), 4, 100);
    pipeline::run(
        &store("split"),
        &input,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    let calls = model.requests.lock().unwrap();
    assert_eq!(calls.len(), 4); // direct, two source halves, one combination
    assert_ne!(
        calls[0]["messages"][1]["content"],
        calls[1]["messages"][1]["content"]
    );
    assert!(calls
        .iter()
        .all(|r| !r.to_string().contains("UNFINISHED RESPONSE")));
}
#[tokio::test]
async fn cancellation_before_save_does_not_replace_cache() {
    let cache = store("cancel");
    let model = Model::start(|r, _| reply(r));
    let input = job(model.base.clone(), 1, 10);
    let old = pipeline::run(&cache, &input, &AtomicBool::new(false), |_, _, _| {})
        .await
        .unwrap();
    let cancel = AtomicBool::new(false);
    assert!(pipeline::run(&cache, &input, &cancel, |s, _, _| {
        if s == "checking_references" {
            cancel.store(true, Ordering::Relaxed)
        }
    })
    .await
    .is_err());
    assert_eq!(cache.load(&input.path).unwrap().markdown, old.markdown);
}

#[tokio::test]
async fn provider_errors_and_single_row_output_limits_never_retry() {
    for provider in ["ollama", "openai", "deepseek"] {
        for limited in [false, true] {
            let model = Model::start(move |_, _| {
                if limited {
                    Reply::limited()
                } else {
                    Reply {
                        status: 401,
                        ..Reply::complete("denied")
                    }
                }
            });
            let mut input = job(model.base.clone(), 1, 10);
            input.settings.provider = provider.into();
            assert!(pipeline::run(
                &store(&format!("error-{provider}-{limited}")),
                &input,
                &AtomicBool::new(false),
                |_, _, _| {}
            )
            .await
            .is_err());
            assert_eq!(model.requests.lock().unwrap().len(), 1);
        }
    }
}
#[tokio::test]
async fn oversized_rows_and_excessive_parts_fail_before_billing() {
    for (count, bytes) in [(1, 200_000), (100, 40_000)] {
        let model = Model::start(|r, _| reply(r));
        let input = job(model.base.clone(), count, bytes);
        assert!(pipeline::run(
            &store(&format!("oversize-{count}")),
            &input,
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .is_err());
        assert!(model.requests.lock().unwrap().is_empty());
    }
}
#[tokio::test]
async fn repeated_truncation_is_bounded_without_saving_partial_text() {
    let model = Model::start(|_, _| Reply::limited());
    let input = job(model.base.clone(), 32, 50);
    let cache = store("always-truncated");
    assert!(
        pipeline::run(&cache, &input, &AtomicBool::new(false), |_, _, _| {})
            .await
            .is_err()
    );
    assert!(model.requests.lock().unwrap().len() <= 4);
    assert!(cache.load(&input.path).is_none());
}
#[tokio::test]
async fn combination_failure_is_not_retried_and_keeps_previous_result() {
    let cache = store("combine-failure");
    let good = Model::start(|r, _| reply(r));
    let old = pipeline::run(
        &cache,
        &job(good.base.clone(), 1, 10),
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    for bad_citation in [false, true] {
        let model = Model::start(move |r, _| {
            if r["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Combine these chronological")
            {
                if bad_citation {
                    Reply::complete("Invented [S999999].")
                } else {
                    Reply::limited()
                }
            } else {
                reply(r)
            }
        });
        assert!(pipeline::run(
            &cache,
            &job(model.base.clone(), 120, 1500),
            &AtomicBool::new(false),
            |_, _, _| {}
        )
        .await
        .is_err());
        assert_eq!(cache.load("/simple.wav").unwrap().markdown, old.markdown);
        assert_eq!(
            model
                .requests
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r["messages"][0]["content"]
                    .as_str()
                    .unwrap()
                    .contains("Combine these chronological"))
                .count(),
            1
        );
    }
}
#[tokio::test]
async fn excerpt_cannot_cite_an_existing_id_outside_its_packet() {
    let model = Model::start(|_, _| Reply::complete("Unsupported [S000120]."));
    let input = job(model.base.clone(), 120, 1500);
    assert!(pipeline::run(
        &store("nonlocal-citation"),
        &input,
        &AtomicBool::new(false),
        |_, _, _| {}
    )
    .await
    .is_err());
    assert_eq!(model.requests.lock().unwrap().len(), 1);
}
#[tokio::test]
async fn cancellation_before_combination_prevents_that_request() {
    let model = Model::start(|r, _| reply(r));
    let input = job(model.base.clone(), 120, 1500);
    let cancel = AtomicBool::new(false);
    assert!(
        pipeline::run(&store("cancel-combine"), &input, &cancel, |s, _, _| {
            if s == "assembling_summary" {
                cancel.store(true, Ordering::Relaxed)
            }
        })
        .await
        .is_err()
    );
    assert!(!model
        .requests
        .lock()
        .unwrap()
        .iter()
        .any(|r| r["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Combine these chronological")));
}
#[tokio::test]
async fn final_combination_never_recursively_reduces_oversized_notes() {
    let model = Model::start(|r, _| {
        let mut reply = reply(r);
        reply.text = format!("{} {}", "long ".repeat(8000), reply.text);
        reply
    });
    let input = job(model.base.clone(), 120, 1500);
    let error = pipeline::run(
        &store("notes-too-long"),
        &input,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap_err();
    assert_eq!(error.to_string(), "summary_context_limit");
    assert!(!model
        .requests
        .lock()
        .unwrap()
        .iter()
        .any(|r| r["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Combine these chronological")));
}
#[tokio::test]
async fn failed_atomic_write_preserves_previous_cache_bytes() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        struct Restore(std::path::PathBuf, std::fs::Permissions);
        impl Drop for Restore {
            fn drop(&mut self) {
                let _ = std::fs::set_permissions(&self.0, self.1.clone());
            }
        }
        let model = Model::start(|r, _| reply(r));
        let input = job(model.base.clone(), 1, 10);
        let cache = store("atomic");
        pipeline::run(&cache, &input, &AtomicBool::new(false), |_, _, _| {})
            .await
            .unwrap();
        let dir = std::env::temp_dir().join(format!("velo-simple-{}-atomic", std::process::id()));
        let path = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|s| s == "json"))
            .unwrap();
        let old = std::fs::read(&path).unwrap();
        std::fs::write(path.with_extension("tmp"), b"scratch").unwrap();
        let restore = Restore(dir.clone(), std::fs::metadata(&dir).unwrap().permissions());
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
        let result = pipeline::run(&cache, &input, &AtomicBool::new(false), |_, _, _| {}).await;
        drop(restore);
        assert!(result.is_err());
        assert_eq!(std::fs::read(&path).unwrap(), old);
    }
}

#[tokio::test]
#[ignore = "requires a local cached transcript; uses loopback only, no cloud or keychain"]
async fn original_transcript_request_shape_is_complete_without_cloud_calls() {
    use velo_lib::transcript::Transcript;
    let transcript: Transcript = serde_json::from_slice(
        &std::fs::read(std::env::var("VELO_TEST_TRANSCRIPT").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(transcript.segments.len(), 2322);
    for format in [
        SummaryFormat::Custom,
        SummaryFormat::Meeting,
        SummaryFormat::KeyPoints,
    ] {
        let model = Model::start(|r, _| reply(r));
        let mut input = job(model.base.clone(), 1, 0);
        input.segments = transcript.segments.clone();
        input.settings.format = format;
        input.settings.language = "th".into();
        let result = pipeline::run(
            &store(&format!("original-local-{format:?}")),
            &input,
            &AtomicBool::new(false),
            |_, _, _| {},
        )
        .await
        .unwrap();
        assert_eq!(result.source_segments, 2322);
        let calls = model.requests.lock().unwrap();
        assert!(
            calls.len() <= 5,
            "original fixture should use few requests, got {}",
            calls.len()
        );
        for row in &input.segments {
            let quoted = serde_json::to_string(&row.text).unwrap();
            assert!(
                calls[..calls.len() - 1]
                    .iter()
                    .any(|r| r["messages"][1]["content"]
                        .as_str()
                        .unwrap()
                        .contains(&quoted)),
                "original source row omitted"
            );
        }
        eprintln!("{format:?}: {} loopback requests; all 2322 source rows supplied, no semantic accuracy claim",calls.len());
    }
}

#[tokio::test]
async fn combination_discards_unsupplied_citations_locally_when_each_block_keeps_a_supplied_one() {
    let model = Model::start(|r, _| {
        if r["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Combine these chronological")
        {
            Reply::complete("Supported condition [S000001] [S999999].")
        } else {
            reply(r)
        }
    });
    let input = job(model.base.clone(), 120, 1500);
    let result = pipeline::run(
        &store("prune-combined-reference"),
        &input,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert!(result.markdown.contains("Supported condition [00:00]"));
    assert!(!result.markdown.contains("S999999"));
    assert_eq!(
        model
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Combine these chronological"))
            .count(),
        1
    );
}

#[tokio::test]
async fn builtin_oversized_notes_are_assembled_locally_without_reduction_or_lost_text() {
    for format in [SummaryFormat::Meeting, SummaryFormat::KeyPoints] {
        let model = Model::start(|r, _| {
            let mut answer = reply(r);
            answer.text = format!("{} {}", "Distinct detail ".repeat(3000), answer.text);
            answer
        });
        let mut input = job(model.base.clone(), 120, 1500);
        input.settings.format = format;
        let result = pipeline::run(
            &store(&format!("local-assembly-{format:?}")),
            &input,
            &AtomicBool::new(false),
            |_, _, _| {},
        )
        .await
        .unwrap();
        assert!(result.markdown.starts_with("## "));
        assert!(result.markdown.contains("### Part 1"));
        let calls = model.requests.lock().unwrap();
        assert_eq!(
            result.markdown.matches("Distinct detail").count(),
            3000 * calls.len()
        );
        assert!(!calls.iter().any(|r| r["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Combine these chronological")));
    }
}

#[tokio::test]
async fn builtin_truncated_combination_keeps_complete_excerpts_without_retrying() {
    let model = Model::start(|r, _| {
        if r["messages"][0]["content"]
            .as_str()
            .unwrap()
            .contains("Combine these chronological")
        {
            Reply::limited()
        } else {
            reply(r)
        }
    });
    let mut input = job(model.base.clone(), 120, 1500);
    input.settings.format = SummaryFormat::Meeting;
    let result = pipeline::run(
        &store("local-after-truncation"),
        &input,
        &AtomicBool::new(false),
        |_, _, _| {},
    )
    .await
    .unwrap();
    assert!(result.markdown.contains("### Part 1"));
    assert!(!result.markdown.contains("UNFINISHED RESPONSE"));
    assert_eq!(
        model
            .requests
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r["messages"][0]["content"]
                .as_str()
                .unwrap()
                .contains("Combine these chronological"))
            .count(),
        1
    );
}
