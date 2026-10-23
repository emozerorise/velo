//! Opt-in cloud acceptance; writes only to a new isolated output directory.
use std::sync::atomic::AtomicBool;
use velo_lib::{
    storage::settings_store::{AppSettings, MeetingDetail, SummaryFormat},
    summary::{credentials::ApiKeyStore, pipeline, JobInput, SummaryStore},
    transcript::Transcript,
};

/// Run each selected variant once; record failures without paid reruns.
#[tokio::test]
#[ignore = "requires explicit API consent and a saved DeepSeek key"]
async fn deepseek_simple_variants() {
    assert_eq!(std::env::var("VELO_DIRECT_LIVE").as_deref(), Ok("deepseek"));
    let transcript: Transcript = serde_json::from_slice(
        &std::fs::read(std::env::var("VELO_TEST_TRANSCRIPT").unwrap()).unwrap(),
    )
    .unwrap();
    let settings: AppSettings = serde_json::from_slice(
        &std::fs::read(std::env::var("VELO_TEST_SETTINGS").unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(transcript.segments.len(), 2322);
    assert_eq!(settings.summary.provider, "deepseek");
    assert_eq!(settings.summary.base_url, "https://api.deepseek.com");
    let build = std::env::var("VELO_TEST_BUILD_HASH").expect("frozen source build hash required");
    let source = std::env::var("VELO_TEST_SOURCE_HASH").expect("verified transcript hash required");
    assert_eq!(
        source,
        "cf41864409f61b2ab126a953e1e3037d1530ac1e4963846400d9b674576dfe3b"
    );
    assert_eq!(build.len(), 64);
    let scope = std::env::var("VELO_DIRECT_VARIANTS").unwrap_or_else(|_| "four".into());
    assert!(["three", "four", "remaining_two"].contains(&scope.as_str()));
    let mut required = if scope == "remaining_two" {
        vec!["meeting", "key_points"]
    } else {
        vec!["custom", "meeting", "key_points"]
    };
    if scope == "four" {
        required.push("meeting_detailed");
    }
    let output = std::path::PathBuf::from(std::env::var("VELO_DIRECT_OUTPUT").unwrap());
    std::fs::create_dir(&output).expect("output must be new");
    let diagnostics = std::fs::File::create(output.join("diagnostics.log")).unwrap();
    let _ = tracing_subscriber::fmt()
        .with_env_filter("velo_lib::summary::generation=debug")
        .with_ansi(false)
        .with_writer(move || diagnostics.try_clone().unwrap())
        .try_init();
    let clocks: Vec<_> = transcript
        .segments
        .iter()
        .map(|row| {
            let seconds = row.start as u64;
            if seconds >= 3600 {
                format!(
                    "{}:{:02}:{:02}",
                    seconds / 3600,
                    seconds / 60 % 60,
                    seconds % 60
                )
            } else {
                format!("{:02}:{:02}", seconds / 60, seconds % 60)
            }
        })
        .collect();
    let mut record = serde_json::json!({"build":build,"source":source,"source_clocks":clocks,"required_variants":required,"cases":{}});
    eprintln!("Reading the existing key through macOS Keychain; no key is logged.");
    let key = ApiKeyStore::for_app("com.velo.player")
        .get(&settings.summary.base_url)
        .unwrap()
        .expect("saved key required");
    let mut failures = Vec::new();
    for (name, format, detail) in [
        ("custom", SummaryFormat::Custom, MeetingDetail::Standard),
        ("meeting", SummaryFormat::Meeting, MeetingDetail::Standard),
        (
            "key_points",
            SummaryFormat::KeyPoints,
            MeetingDetail::Standard,
        ),
        (
            "meeting_detailed",
            SummaryFormat::Meeting,
            MeetingDetail::Detailed,
        ),
    ] {
        if !required.contains(&name) {
            continue;
        }
        let offset = std::fs::metadata(output.join("diagnostics.log"))
            .unwrap()
            .len() as usize;
        let mut selected = settings.summary.clone();
        selected.format = format;
        selected.meeting_detail = detail;
        if format == SummaryFormat::Custom {
            selected.instructions =
                "สรุปเป็นย่อหน้าเดียว ไม่ต้องมีหัวข้อหรือรายการ เก็บประเด็นสำคัญและเงื่อนไขให้ครบ".into();
        }
        let input = JobInput {
            path: transcript.path.clone(),
            segments: transcript.segments.clone(),
            settings: selected,
            vocabulary: settings.transcript.prompt.clone(),
            transcript_language: transcript.language.clone(),
            api_key: Some(key.clone()),
        };
        let store = SummaryStore::open(output.join(name)).unwrap();
        eprintln!("Starting {name}");
        let result = pipeline::run(
            &store,
            &input,
            &AtomicBool::new(false),
            |stage, done, total| eprintln!("{name}: {stage} {done}/{total}"),
        )
        .await;
        let log = std::fs::read(output.join("diagnostics.log")).unwrap();
        let calls = String::from_utf8_lossy(&log[offset..])
            .lines()
            .filter_map(|line| {
                line.split("physical_call=")
                    .nth(1)?
                    .split_whitespace()
                    .next()?
                    .parse::<usize>()
                    .ok()
            })
            .max()
            .unwrap_or(0);
        let mut item = serde_json::json!({"build":build,"source":source,"calls":calls});
        match result {
            Ok(summary) => {
                let shape_ok = format != SummaryFormat::Custom
                    || (!summary.markdown.trim().contains('\n')
                        && !summary.markdown.trim_start().starts_with('#'));
                item["outcome"] = serde_json::json!(if shape_ok { "complete" } else { "failed" });
                item["summary"] = serde_json::to_value(&summary).unwrap();
                std::fs::write(output.join(format!("{name}.md")), &summary.markdown).unwrap();
                std::fs::write(
                    output.join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&summary).unwrap(),
                )
                .unwrap();
                record["cases"][name] = item;
                std::fs::write(
                    output.join("run.json"),
                    serde_json::to_vec_pretty(&record).unwrap(),
                )
                .unwrap();
                if !shape_ok {
                    failures.push(name);
                }
                eprintln!(
                    "{name}: generated in {calls} requests; semantic source comparison pending"
                );
            }
            Err(error) => {
                item["outcome"] = serde_json::json!("failed");
                item["error"] = serde_json::json!(error.to_string());
                record["cases"][name] = item;
                std::fs::write(
                    output.join("run.json"),
                    serde_json::to_vec_pretty(&record).unwrap(),
                )
                .unwrap();
                failures.push(name);
                eprintln!("{name} failed: {error}; no retry of this variant");
            }
        }
    }
    assert!(failures.is_empty(), "Failed variants: {failures:?}");
}

#[tokio::test]
#[ignore = "requires explicit API consent and a saved DeepSeek key"]
async fn deepseek_four_variants() {
    run_variants(false).await;
}

#[tokio::test]
#[ignore = "requires explicit API consent and a separately accepted detailed result"]
async fn deepseek_remaining_variants() {
    run_variants(true).await;
}

async fn run_variants(skip_detailed: bool) {
    // Only size/phase fields are emitted; no messages, headers or credentials.
    let _ = tracing_subscriber::fmt()
        .with_env_filter("velo_lib::summary::pipeline=debug")
        .with_ansi(false)
        .try_init();
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
    // Prove the new detailed path first; stop billing if any variant fails.
    for (name, format, detail) in [
        (
            "meeting_detailed",
            SummaryFormat::Meeting,
            MeetingDetail::Detailed,
        ),
        ("custom", SummaryFormat::Custom, MeetingDetail::Standard),
        ("meeting", SummaryFormat::Meeting, MeetingDetail::Standard),
        (
            "key_points",
            SummaryFormat::KeyPoints,
            MeetingDetail::Standard,
        ),
    ] {
        if skip_detailed && detail == MeetingDetail::Detailed {
            eprintln!(
                "meeting_detailed: separately accepted; this run covers the remaining three only"
            );
            continue;
        }
        let mut s = settings.summary.clone();
        s.format = format;
        s.meeting_detail = detail;
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
                assert_eq!(summary.meeting_detail, detail);
                std::fs::write(output.join(format!("{name}.md")), &summary.markdown).unwrap();
                std::fs::write(
                    output.join(format!("{name}.json")),
                    serde_json::to_vec_pretty(&summary).unwrap(),
                )
                .unwrap();
                if format == SummaryFormat::Custom {
                    assert_eq!(
                        summary
                            .markdown
                            .trim()
                            .split("\n\n")
                            .filter(|part| !part.trim().is_empty())
                            .count(),
                        1,
                        "custom prompt requires exactly one paragraph; stop before billing other formats"
                    );
                    assert!(!summary.markdown.lines().any(|line| {
                        let line = line.trim_start();
                        line.starts_with('#')
                            || line.starts_with("- ")
                            || line.starts_with("* ")
                            || line.starts_with("+ ")
                    }));
                }
                eprintln!("{name}: generated; requires human source comparison");
            }
            Err(e) => {
                std::fs::write(output.join(format!("{name}-error.txt")), e.to_string()).unwrap();
                failures.push(name);
                break;
            }
        }
    }
    assert!(failures.is_empty(), "Failed formats: {failures:?}");
}
