//! whisper.cpp runner.
//!
//! PROTOTYPE NOTE: this shells out to the `whisper-cli` binary and to a model
//! file the user supplies, which keeps the app build light while the feature
//! is being evaluated. A shipping version should link `whisper-rs` in-process
//! and download the model on first use, so releases stay self-contained.

use crate::errors::{Result, VeloError};
use crate::transcript::types::{EngineStatus, TranscriptSegment};
use serde::Deserialize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const BIN_ENV: &str = "VELO_WHISPER_BIN";
const MODEL_ENV: &str = "VELO_WHISPER_MODEL";

/// `whisper-cli`, from the env override, the app's own model folder, or PATH.
pub fn resolve_binary(app_data_dir: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(BIN_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let bundled = app_data_dir.join("whisper").join("whisper-cli");
    if bundled.is_file() {
        return Some(bundled);
    }

    let path_var = std::env::var("PATH").ok()?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join("whisper-cli"))
        .find(|candidate| candidate.is_file())
}

/// A ggml model file: the env override, else the first `.bin` in the app's
/// `whisper/` folder.
pub fn resolve_model(app_data_dir: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(MODEL_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let dir = app_data_dir.join("whisper");
    let mut models: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("bin"))
        .collect();
    models.sort();
    models.into_iter().next()
}

pub fn status(app_data_dir: &Path) -> EngineStatus {
    let whisper_bin = resolve_binary(app_data_dir);
    let model_path = resolve_model(app_data_dir);
    EngineStatus {
        ready: whisper_bin.is_some() && model_path.is_some(),
        whisper_bin: whisper_bin.map(|p| p.to_string_lossy().into_owned()),
        model_path: model_path.map(|p| p.to_string_lossy().into_owned()),
    }
}

#[derive(Deserialize)]
struct WhisperOutput {
    result: WhisperResult,
    transcription: Vec<WhisperSegment>,
}

#[derive(Deserialize)]
struct WhisperResult {
    language: String,
}

#[derive(Deserialize)]
struct WhisperSegment {
    offsets: WhisperOffsets,
    text: String,
}

#[derive(Deserialize)]
struct WhisperOffsets {
    from: i64,
    to: i64,
}

/// Transcribe a 16 kHz mono WAV. Returns the detected language and segments.
///
/// `prompt` is optional domain vocabulary; whisper conditions on it, which is
/// what keeps English product terms in a Thai meeting from coming back as
/// Thai phonetic spellings.
///
/// `on_progress` receives 0.0..1.0 as whisper reports it.
#[allow(clippy::too_many_arguments)]
pub fn transcribe(
    bin: &Path,
    model: &Path,
    wav: &Path,
    language: &str,
    prompt: &str,
    cancel: &Arc<AtomicBool>,
    on_progress: &dyn Fn(f64),
) -> Result<(String, Vec<TranscriptSegment>)> {
    let prefix = wav.with_extension("");
    let json_path = wav.with_extension("json");
    let threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1))
        .unwrap_or(4);

    let mut command = Command::new(bin);
    command
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(wav)
        .arg("-l")
        .arg(language)
        .arg("-t")
        .arg(threads.to_string())
        .arg("-oj")
        .arg("-of")
        .arg(&prefix)
        // Progress percentages, which are the only stderr output parsed below.
        .arg("-pp");

    if !prompt.trim().is_empty() {
        // Without carrying it, the prompt only conditions the first 30-second
        // window -- and a meeting's vocabulary matters just as much an hour in.
        command
            .arg("--prompt")
            .arg(prompt.trim())
            .arg("--carry-initial-prompt");
    }

    let mut child = command
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| VeloError::Player(format!("Could not start whisper-cli: {}", e)))?;

    if let Some(stderr) = child.stderr.take() {
        for line in BufReader::new(stderr)
            .lines()
            .map_while(std::result::Result::ok)
        {
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                return Err(VeloError::Player("Transcription cancelled".into()));
            }

            if let Some(rest) = line.split("progress =").nth(1) {
                if let Ok(percent) = rest.trim().trim_end_matches('%').trim().parse::<f64>() {
                    on_progress((percent / 100.0).clamp(0.0, 1.0));
                }
            }
        }
    }

    let exit = child
        .wait()
        .map_err(|e| VeloError::Player(format!("whisper-cli failed: {}", e)))?;
    if !exit.success() {
        return Err(VeloError::Player(format!(
            "whisper-cli exited with {}",
            exit
        )));
    }

    let content = std::fs::read_to_string(&json_path)
        .map_err(|e| VeloError::Player(format!("Could not read whisper output: {}", e)))?;
    let output: WhisperOutput = serde_json::from_str(&content)
        .map_err(|e| VeloError::Player(format!("Could not parse whisper output: {}", e)))?;
    let _ = std::fs::remove_file(&json_path);

    let segments = output
        .transcription
        .into_iter()
        .map(|s| TranscriptSegment {
            start: s.offsets.from as f64 / 1000.0,
            end: s.offsets.to as f64 / 1000.0,
            text: s.text.trim().to_string(),
        })
        .filter(|s| !s.text.is_empty())
        .collect();

    Ok((output.result.language, segments))
}
