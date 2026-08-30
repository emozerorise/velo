//! End-to-end check of the transcription pipeline against a real media file.
//!
//! Both tests skip themselves unless pointed at real inputs, so a checkout
//! without a sample file (or without the model downloaded) still runs green:
//!
//! ```sh
//! VELO_TEST_MEDIA=/path/to/clip.mp4 \
//! VELO_WHISPER_MODEL=/path/to/ggml-large-v3-turbo-q5_0.bin \
//! cargo test --test transcript_pipeline -- --nocapture
//! ```

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use velo_lib::transcript::{audio, engine, model};

fn sample_media() -> Option<String> {
    let path = std::env::var("VELO_TEST_MEDIA").ok()?;
    PathBuf::from(&path).is_file().then_some(path)
}

fn wav_header(bytes: &[u8]) -> (u16, u32) {
    // Channel count and sample rate live at fixed offsets in the fmt chunk.
    let channels = u16::from_le_bytes([bytes[22], bytes[23]]);
    let sample_rate = u32::from_le_bytes([bytes[24], bytes[25], bytes[26], bytes[27]]);
    (channels, sample_rate)
}

#[test]
fn extracts_16khz_mono_wav() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let dest = std::env::temp_dir().join("velo-extract-test.wav");
    let cancel = Arc::new(AtomicBool::new(false));
    let last_progress = Cell::new(-1.0_f64);

    audio::extract(&media, &dest, &cancel, &|p| {
        if p >= 0.0 {
            assert!(p >= last_progress.get() - 0.01, "progress went backwards");
            last_progress.set(p);
        }
    })
    .expect("extraction failed");

    let bytes = std::fs::read(&dest).expect("no wav written");
    assert!(bytes.len() > 44, "wav has no audio data");
    assert_eq!(&bytes[0..4], b"RIFF");

    let (channels, sample_rate) = wav_header(&bytes);
    assert_eq!(channels, 1, "whisper needs mono");
    assert_eq!(sample_rate, 16_000, "whisper needs 16 kHz");

    // The reader has to agree with what the extractor wrote, or transcription
    // gets silence or noise instead of speech.
    let samples = audio::read_samples(&dest).expect("could not read samples back");
    assert!(
        samples.len() > 16_000,
        "expected at least a second of audio, got {} samples",
        samples.len()
    );
    assert!(
        samples.iter().any(|s| s.abs() > 0.01),
        "every sample is silent"
    );

    let _ = std::fs::remove_file(&dest);
}

#[test]
fn transcribes_extracted_audio() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let Some(model_path) = model::resolve(&std::env::temp_dir()) else {
        eprintln!("skipping: no whisper model available");
        return;
    };

    let wav = std::env::temp_dir().join("velo-transcribe-test.wav");
    let cancel = Arc::new(AtomicBool::new(false));
    audio::extract(&media, &wav, &cancel, &|_| {}).expect("extraction failed");
    let samples = audio::read_samples(&wav).expect("could not read samples");

    // Both branches: without a vocabulary prompt, and with one.
    for prompt in [
        "",
        "ระบบ HR, master data, employee, dashboard, group section",
    ] {
        let seen_progress = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = seen_progress.clone();
        let on_progress: engine::ProgressFn = Arc::new(move |_| {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        });

        let (language, segments) =
            engine::transcribe(&model_path, &samples, "auto", prompt, &cancel, on_progress)
                .expect("transcription failed");

        assert!(!language.is_empty(), "no language reported");
        assert!(!segments.is_empty(), "no segments produced");
        assert!(
            seen_progress.load(std::sync::atomic::Ordering::Relaxed),
            "progress was never reported"
        );

        for pair in segments.windows(2) {
            assert!(
                pair[1].start >= pair[0].start,
                "segments are out of order: {:?} then {:?}",
                pair[0],
                pair[1]
            );
        }
        assert!(
            segments.iter().all(|s| s.end >= s.start),
            "a segment ends before it starts"
        );

        let audio_seconds = samples.len() as f64 / 16_000.0;
        let last_end = segments.last().expect("no segments").end;
        assert!(
            last_end <= audio_seconds + 1.0,
            "a timestamp ran past the end of the audio ({last_end:.1}s of {audio_seconds:.1}s)"
        );

        eprintln!(
            "prompt {:?} -> {} segments ({}), first: [{:.1}s] {}",
            prompt,
            segments.len(),
            language,
            segments[0].start,
            segments[0].text
        );
    }

    let _ = std::fs::remove_file(&wav);
}

/// The chunking path is what reproduces `--carry-initial-prompt`, and it is
/// also where timestamps could silently drift: a segment from the second
/// chunk is timed against that chunk unless the offset is added back.
#[test]
fn segments_past_the_first_chunk_keep_media_timestamps() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let Some(model_path) = model::resolve(&std::env::temp_dir()) else {
        eprintln!("skipping: no whisper model available");
        return;
    };

    let wav = std::env::temp_dir().join("velo-chunk-test.wav");
    let cancel = Arc::new(AtomicBool::new(false));
    audio::extract(&media, &wav, &cancel, &|_| {}).expect("extraction failed");
    let samples = audio::read_samples(&wav).expect("could not read samples");

    let chunk_seconds = 60;
    let audio_seconds = samples.len() as f64 / 16_000.0;
    assert!(
        audio_seconds > chunk_seconds as f64 * 1.5,
        "fixture is too short to span chunks ({audio_seconds:.0}s)"
    );

    let (_, segments) = engine::transcribe_chunked(
        &model_path,
        &samples,
        "auto",
        "",
        &cancel,
        Arc::new(|_| {}),
        chunk_seconds,
    )
    .expect("transcription failed");

    let last_end = segments.last().expect("no segments").end;
    assert!(
        last_end > chunk_seconds as f64,
        "every timestamp landed inside the first chunk, so offsets are lost \
         (last segment ends at {last_end:.1}s)"
    );
    assert!(
        last_end <= audio_seconds + 1.0,
        "a timestamp ran past the end of the audio ({last_end:.1}s of {audio_seconds:.1}s)"
    );

    for pair in segments.windows(2) {
        assert!(
            pair[1].start >= pair[0].start,
            "chunk boundary broke ordering: {:?} then {:?}",
            pair[0],
            pair[1]
        );
    }

    let _ = std::fs::remove_file(&wav);
}

#[test]
fn cancelling_stops_transcription() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let Some(model_path) = model::resolve(&std::env::temp_dir()) else {
        eprintln!("skipping: no whisper model available");
        return;
    };

    let wav = std::env::temp_dir().join("velo-cancel-test.wav");
    let cancel = Arc::new(AtomicBool::new(false));
    audio::extract(&media, &wav, &cancel, &|_| {}).expect("extraction failed");
    let samples = audio::read_samples(&wav).expect("could not read samples");

    // Already cancelled before the first window: whisper's abort callback has
    // to be honoured, or a cancelled eight-hour job runs to the end anyway.
    cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let result = engine::transcribe(&model_path, &samples, "auto", "", &cancel, Arc::new(|_| {}));

    assert!(result.is_err(), "a cancelled run should not succeed");

    let _ = std::fs::remove_file(&wav);
}
