//! Archived extractive acceptance. Use summary_direct_live for the current runner.
#![cfg(feature = "summary-evidence-qa")]
#[path = "support/evidence_acceptance.rs"]
mod evidence_acceptance;
use std::sync::atomic::AtomicBool;
use velo_lib::storage::settings_store::{AppSettings, SummaryFormat};
use velo_lib::summary::{
    credentials::ApiKeyStore, evidence_pipeline as pipeline, JobInput, SummaryStore,
};
use velo_lib::transcript::Transcript;

#[tokio::test]
#[ignore = "requires explicit consent, a transcript, saved DeepSeek key and a new output directory"]
async fn live_deepseek_all_three_formats() {
    assert_eq!(
        std::env::var("VELO_FIDELITY_LIVE").as_deref(),
        Ok("deepseek")
    );
    let source_path = std::env::var("VELO_TEST_TRANSCRIPT").expect("transcript path required");
    let settings_path = std::env::var("VELO_TEST_SETTINGS").expect("settings path required");
    let output = std::path::PathBuf::from(
        std::env::var("VELO_FIDELITY_OUTPUT").expect("new output directory required"),
    );
    let transcript: Transcript =
        serde_json::from_str(&std::fs::read_to_string(source_path).unwrap()).unwrap();
    let settings: AppSettings =
        serde_json::from_str(&std::fs::read_to_string(settings_path).unwrap()).unwrap();
    assert_eq!(settings.summary.provider, "deepseek");
    assert_eq!(settings.summary.base_url, "https://api.deepseek.com");
    std::fs::create_dir(&output).expect("output must not already exist");
    let key = ApiKeyStore::for_app("com.velo.player")
        .get(&settings.summary.base_url)
        .expect("could not read the saved key")
        .expect("no saved DeepSeek key");
    let mut failures = Vec::new();
    for (name, format) in [
        ("meeting", SummaryFormat::Meeting),
        ("key-points", SummaryFormat::KeyPoints),
        ("custom", SummaryFormat::Custom),
    ] {
        let mut summary_settings = settings.summary.clone();
        summary_settings.format = format;
        if format == SummaryFormat::Custom {
            summary_settings.instructions =
                "สรุปเป็นย่อหน้าเดียว ไม่ต้องมีหัวข้อหรือรายการ เก็บประเด็นสำคัญและเงื่อนไขให้ครบ".into();
        }
        let store = SummaryStore::open(output.join(name)).unwrap();
        let input = JobInput {
            path: transcript.path.clone(),
            segments: transcript.segments.clone(),
            transcript_language: transcript.language.clone(),
            vocabulary: settings.transcript.prompt.clone(),
            settings: summary_settings,
            api_key: Some(key.clone()),
        };
        eprintln!("Running {name} through the production pipeline");
        let artifacts =
            evidence_acceptance::Artifacts::new(output.join(format!("{name}-artifacts")));
        let result = pipeline::run_observed(
            &store,
            &input,
            &AtomicBool::new(false),
            |stage, done, total| {
                eprintln!("{name}: {stage} {done}/{total}");
            },
            &artifacts,
        )
        .await;
        let manifest = artifacts.manifest();
        std::fs::write(
            output.join(format!("{name}-manifest.json")),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert!(manifest["requests"].as_u64().unwrap() <= manifest["ceiling"].as_u64().unwrap());
        match result {
            Ok(summary) => {
                std::fs::write(
                    output.join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&summary).unwrap(),
                )
                .unwrap();
                let labels: std::collections::BTreeSet<_> = transcript
                    .segments
                    .iter()
                    .map(|s| velo_lib::summary::chunk::stamp(s.start))
                    .collect();
                for label in evidence_acceptance::application_labels(&summary.markdown) {
                    assert!(
                        labels.contains(label),
                        "invalid application reference {label}"
                    );
                }
                if format == SummaryFormat::Custom {
                    assert!(evidence_acceptance::is_plain_paragraph(&summary.markdown));
                }
                eprintln!("{name}: saved for human source comparison");
            }
            Err(error) => {
                // No API key or request headers enter the report.
                std::fs::write(output.join(format!("{name}-error.txt")), error.to_string())
                    .unwrap();
                failures.push(name);
                eprintln!("{name}: failed validation or provider request; see local error report");
            }
        }
    }
    assert!(
        failures.is_empty(),
        "formats did not produce checked output: {failures:?}"
    );
}
