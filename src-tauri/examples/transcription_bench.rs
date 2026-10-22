//! Local-only timing of the production extraction and transcription pipeline.
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use velo_lib::transcript::{audio, engine};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: transcription_bench MEDIA MODEL cpu|gpu OUTPUT.json".into());
    }
    let media = &args[0];
    let model = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[3]);
    if output.exists() {
        return Err("refusing to overwrite an existing benchmark result".into());
    }
    let device = match args[2].as_str() {
        "cpu" => engine::TranscriptionDevice::Cpu,
        "gpu" => engine::TranscriptionDevice::Auto,
        _ => return Err("device must be cpu or gpu".into()),
    };
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .with_writer(std::io::stderr)
        .init();
    whisper_rs::install_logging_hooks();
    let started = Instant::now();
    let gpu_setup_started = Instant::now();
    let gpu_names = available_gpu_names(&args[2]);
    let gpu_setup_seconds = gpu_setup_started.elapsed().as_secs_f64();
    if args[2] == "gpu" && gpu_names.is_empty() {
        return Err(
            "no Vulkan GPU found; refusing to label a CPU fallback as a GPU benchmark".into(),
        );
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let wav = output.with_extension("wav");
    if wav.exists() {
        return Err("refusing to overwrite existing benchmark audio".into());
    }
    let extraction_started = Instant::now();
    audio::extract(media, &wav, &cancel, &|_| {})?;
    let samples = audio::read_samples(&wav)?;
    std::fs::remove_file(&wav)?;
    let extraction_seconds = extraction_started.elapsed().as_secs_f64();
    let audio_seconds = samples.len() as f64 / 16_000.0;
    let last_percent = Arc::new(AtomicUsize::new(usize::MAX));
    let inference_started = Instant::now();
    let (language, segments) = engine::transcribe_with_device(
        &model,
        &samples,
        "auto",
        "",
        &cancel,
        Arc::new(move |progress| {
            let percent = (progress * 100.0).round() as usize;
            if last_percent.swap(percent, Ordering::Relaxed) != percent {
                eprintln!(
                    "transcription {percent}% at {:.1}s",
                    inference_started.elapsed().as_secs_f64()
                );
            }
        }),
        device,
    )?;
    let transcription_seconds = inference_started.elapsed().as_secs_f64();
    let total_seconds = started.elapsed().as_secs_f64();
    if language.is_empty() || segments.is_empty() {
        return Err("transcription produced no language or segments".into());
    }
    if segments.iter().any(|segment| {
        !segment.start.is_finite()
            || !segment.end.is_finite()
            || segment.start < 0.0
            || segment.end < segment.start
            || segment.end > audio_seconds + 1.0
    }) || segments
        .windows(2)
        .any(|pair| pair[1].start < pair[0].start)
    {
        return Err("transcription has invalid or unordered timestamps".into());
    }
    let report = serde_json::json!({
        "device": args[2],
        "gpu_names": gpu_names,
        "model": model.file_name().and_then(|name| name.to_str()),
        "audio_seconds": audio_seconds,
        "gpu_setup_seconds": gpu_setup_seconds,
        "extraction_seconds": extraction_seconds,
        "transcription_seconds": transcription_seconds,
        "total_seconds": total_seconds,
        "realtime_factor": transcription_seconds / audio_seconds,
        "language": language,
        "prompt": "",
        "beam_size": 5,
        "segments": segments,
        "system_info": whisper_rs::print_system_info(),
    });
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
    eprintln!("Completed in {total_seconds:.2}s ({transcription_seconds:.2}s transcription) for {audio_seconds:.3}s audio");
    Ok(())
}

fn available_gpu_names(device: &str) -> Vec<String> {
    #[cfg(target_os = "windows")]
    if device == "gpu" {
        return whisper_rs::vulkan::list_devices()
            .into_iter()
            .map(|gpu| gpu.name)
            .collect();
    }
    let _ = device;
    Vec::new()
}
