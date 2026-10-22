//! Local-only timing of the production extraction and transcription pipeline.
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use velo_lib::transcript::{audio, engine};

fn main() -> Result<(), Box<dyn Error>> {
    if velo_lib::transcript::worker::run_if_requested() {
        return Ok(());
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(4..=5).contains(&args.len()) {
        return Err("usage: transcription_bench MEDIA MODEL cpu|gpu|auto|gpu-fallback OUTPUT.json [MAX_SECONDS]".into());
    }
    let media = &args[0];
    let model = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[3]);
    if output.exists() {
        return Err("refusing to overwrite an existing benchmark result".into());
    }
    let device = match args[2].as_str() {
        "cpu" => engine::TranscriptionDevice::Cpu,
        "gpu" | "gpu-fallback" => engine::TranscriptionDevice::Gpu,
        "auto" => engine::TranscriptionDevice::Auto,
        _ => return Err("device must be cpu, gpu, auto or gpu-fallback".into()),
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
    let mut samples = audio::read_samples(&wav)?;
    if let Some(limit) = args.get(4) {
        let seconds: f64 = limit.parse()?;
        if !seconds.is_finite() || seconds <= 0.0 {
            return Err("MAX_SECONDS must be positive and finite".into());
        }
        samples.truncate((seconds * 16_000.0) as usize);
    }
    std::fs::remove_file(&wav)?;
    let extraction_seconds = extraction_started.elapsed().as_secs_f64();
    let audio_seconds = samples.len() as f64 / 16_000.0;
    let last_percent = Arc::new(AtomicUsize::new(usize::MAX));
    let inference_started = Instant::now();
    let execution = Arc::new(Mutex::new(None));
    let on_progress: engine::ProgressFn = Arc::new(move |progress| {
        let percent = (progress * 100.0).round() as usize;
        if last_percent.swap(percent, Ordering::Relaxed) != percent {
            eprintln!(
                "transcription {percent}% at {:.1}s",
                inference_started.elapsed().as_secs_f64()
            );
        }
    });
    let (language, segments) = if args[2] == "gpu" {
        engine::transcribe_with_device(&model, &samples, "auto", "", &cancel, on_progress, device)?
    } else {
        let captured = execution.clone();
        engine::transcribe_with_fallback(
            &model,
            &samples,
            "auto",
            "",
            &cancel,
            on_progress,
            device,
            Arc::new(move |status| {
                eprintln!(
                    "processor: {:?}, fallback: {:?}",
                    status.device, status.fallback
                );
                *captured.lock().unwrap() = Some(status);
            }),
        )?
    };
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
        "execution": *execution.lock().unwrap(),
        "max_seconds": args.get(4),
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
    if device != "gpu" || !engine::gpu_available() {
        return Vec::new();
    }
    use whisper_rs::whisper_rs_sys as sys;
    // SAFETY: preflight initialized the registry; its device names are static.
    unsafe {
        (0..sys::ggml_backend_dev_count())
            .filter_map(|index| {
                let dev = sys::ggml_backend_dev_get(index);
                if !matches!(
                    sys::ggml_backend_dev_type(dev),
                    sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_GPU
                        | sys::ggml_backend_dev_type_GGML_BACKEND_DEVICE_TYPE_IGPU
                ) {
                    return None;
                }
                Some(
                    std::ffi::CStr::from_ptr(sys::ggml_backend_dev_name(dev))
                        .to_string_lossy()
                        .into_owned(),
                )
            })
            .collect()
    }
}
