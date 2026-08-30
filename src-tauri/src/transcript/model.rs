//! The whisper model: where it lives, and fetching it on first use.
//!
//! The model is far too large to ship inside the app bundle, so a release
//! carries the engine but not its weights and downloads them once, on demand,
//! into the app's data directory.

use crate::errors::{Result, VeloError};
use futures_util::StreamExt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

/// Multilingual, quantised, and the smallest model that transcribes Thai
/// mixed with English well enough to be worth reading.
pub const MODEL_FILE: &str = "ggml-large-v3-turbo-q5_0.bin";
pub const MODEL_BYTES: u64 = 574_041_195;

const MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin";

/// Anything much smaller than the real file is a truncated or error response
/// rather than a model, and whisper would only fail later and less clearly.
const MIN_PLAUSIBLE_BYTES: u64 = 400 * 1024 * 1024;

const MODEL_ENV: &str = "VELO_WHISPER_MODEL";

pub fn model_dir(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("whisper")
}

/// The model to load: an explicit override, the downloaded copy, or nothing.
pub fn resolve(app_data_dir: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var(MODEL_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let downloaded = model_dir(app_data_dir).join(MODEL_FILE);
    downloaded.is_file().then_some(downloaded)
}

/// Stream the model to disk, reporting `(received, total)` as it goes.
///
/// Downloads into a `.part` file and renames only on success, so an
/// interrupted download can never be mistaken for a usable model.
pub async fn download(
    app_data_dir: &Path,
    cancel: Arc<AtomicBool>,
    on_progress: impl Fn(u64, u64),
) -> Result<PathBuf> {
    let dir = model_dir(app_data_dir);
    std::fs::create_dir_all(&dir)
        .map_err(|e| VeloError::Storage(format!("Could not create model dir: {}", e)))?;

    let final_path = dir.join(MODEL_FILE);
    let part_path = dir.join(format!("{}.part", MODEL_FILE));

    info!("downloading whisper model to {}", part_path.display());

    let response = reqwest::get(MODEL_URL)
        .await
        .map_err(|e| VeloError::Player(format!("Could not reach the model host: {}", e)))?;

    if !response.status().is_success() {
        return Err(VeloError::Player(format!(
            "Model download failed with HTTP {}",
            response.status()
        )));
    }

    let total = response.content_length().unwrap_or(MODEL_BYTES);
    let mut file = std::fs::File::create(&part_path)
        .map_err(|e| VeloError::Storage(format!("Could not write the model file: {}", e)))?;

    let mut received: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::Relaxed) {
            drop(file);
            let _ = std::fs::remove_file(&part_path);
            return Err(VeloError::Player("Model download cancelled".into()));
        }

        let chunk =
            chunk.map_err(|e| VeloError::Player(format!("Model download interrupted: {}", e)))?;
        file.write_all(&chunk)
            .map_err(|e| VeloError::Storage(format!("Could not write the model file: {}", e)))?;

        received += chunk.len() as u64;
        on_progress(received, total);
    }

    file.flush()
        .map_err(|e| VeloError::Storage(format!("Could not finish the model file: {}", e)))?;
    drop(file);

    if received < MIN_PLAUSIBLE_BYTES {
        let _ = std::fs::remove_file(&part_path);
        return Err(VeloError::Player(format!(
            "The download stopped early ({} of {} bytes)",
            received, total
        )));
    }

    std::fs::rename(&part_path, &final_path)
        .map_err(|e| VeloError::Storage(format!("Could not save the model: {}", e)))?;

    info!("whisper model ready at {}", final_path.display());
    Ok(final_path)
}
