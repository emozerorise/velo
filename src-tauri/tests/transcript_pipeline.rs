//! End-to-end check of the transcription pipeline against a real media file.
//!
//! Both tests skip themselves unless pointed at real inputs, so a checkout
//! without a sample file (or without whisper installed) still runs green:
//!
//! ```sh
//! VELO_TEST_MEDIA=/path/to/clip.mp4 \
//! VELO_WHISPER_BIN=$(which whisper-cli) \
//! VELO_WHISPER_MODEL=/path/to/ggml-large-v3-turbo-q5_0.bin \
//! cargo test --test transcript_pipeline -- --nocapture
//! ```

use std::cell::Cell;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use velo_lib::transcript::{audio, engine};

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

    let _ = std::fs::remove_file(&dest);
}

#[test]
fn transcribes_extracted_audio() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let app_data = std::env::temp_dir();
    let (Some(bin), Some(model)) = (
        engine::resolve_binary(&app_data),
        engine::resolve_model(&app_data),
    ) else {
        eprintln!("skipping: whisper binary or model not available");
        return;
    };

    let wav = std::env::temp_dir().join("velo-transcribe-test.wav");
    let cancel = Arc::new(AtomicBool::new(false));
    audio::extract(&media, &wav, &cancel, &|_| {}).expect("extraction failed");

    // Both branches: without a prompt, and with one -- the latter adds
    // `--prompt` and `--carry-initial-prompt` to whisper's command line.
    for prompt in [
        "",
        "ระบบ HR, master data, employee, dashboard, group section",
    ] {
        let (language, segments) =
            engine::transcribe(&bin, &model, &wav, "auto", prompt, &cancel, &|_| {})
                .expect("transcription failed");

        assert!(!language.is_empty(), "no language reported");
        assert!(!segments.is_empty(), "no segments produced");

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
