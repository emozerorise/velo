//! Opt-in cloud acceptance; writes only to a new isolated output directory.
use std::sync::atomic::AtomicBool;
use velo_lib::{
    storage::settings_store::{AppSettings, SummaryFormat},
    summary::{credentials::ApiKeyStore, pipeline, JobInput, SummaryStore},
    transcript::Transcript,
};

#[tokio::test]
#[ignore = "requires explicit API consent and a saved DeepSeek key"]
async fn deepseek_three_formats() {
    assert_eq!(std::env::var("VELO_DIRECT_LIVE").as_deref(), Ok("deepseek"));
    let transcript: Transcript = serde_json::from_slice(
        &std::fs::read(std::env::var("VELO_TEST_TRANSCRIPT").unwrap()).unwrap(),
    )
    .unwrap();
    let settings: AppSettings = serde_json::from_slice(
        &std::fs::read(std::env::var("VELO_TEST_SETTINGS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(settings.summary.provider, "deepseek");
    assert_eq!(settings.summary.base_url, "https://api.deepseek.com");
    let output = std::path::PathBuf::from(std::env::var("VELO_DIRECT_OUTPUT").unwrap());
    std::fs::create_dir(&output).expect("output must be new");
    eprintln!("Reading the existing key through macOS Keychain; no key is logged.");
    let key = ApiKeyStore::for_app("com.velo.player")
        .get(&settings.summary.base_url)
        .unwrap()
        .expect("saved key required");
    let mut failures = Vec::new();
    for (name, format) in [
        ("meeting", SummaryFormat::Meeting),
        ("key_points", SummaryFormat::KeyPoints),
        ("custom", SummaryFormat::Custom),
    ] {
        let mut s = settings.summary.clone();
        s.format = format;
        if format == SummaryFormat::Custom {
            s.instructions =
                "สรุปเป็นย่อหน้าเดียว ไม่ต้องมีหัวข้อหรือรายการ เก็บประเด็นสำคัญและเงื่อนไขให้ครบ".into();
        }
        let input = JobInput {
            path: transcript.path.clone(),
            segments: transcript.segments.clone(),
            settings: s,
            vocabulary: settings.transcript.prompt.clone(),
            transcript_language: transcript.language.clone(),
            api_key: Some(key.clone()),
        };
        let store = SummaryStore::open(output.join(name)).unwrap();
        eprintln!("Starting {name}");
        match pipeline::run(
            &store,
            &input,
            &AtomicBool::new(false),
            |stage, done, total| eprintln!("{name}: {stage} {done}/{total}"),
        )
        .await
        {
            Ok(summary) => {
                std::fs::write(output.join(format!("{name}.md")), &summary.markdown).unwrap();
                std::fs::write(
                    output.join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&summary).unwrap(),
                )
                .unwrap();
                eprintln!("{name}: generated; requires human source comparison");
            }
            Err(e) => {
                std::fs::write(output.join(format!("{name}-error.txt")), e.to_string()).unwrap();
                failures.push(name);
            }
        }
    }
    assert!(failures.is_empty(), "Failed formats: {failures:?}");
}
