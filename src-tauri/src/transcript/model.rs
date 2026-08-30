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

/// The copy the app downloaded and therefore owns.
///
/// An override from `VELO_WHISPER_MODEL` points at a file somewhere on the
/// user's disk that the app did not put there, so it is deliberately excluded:
/// removing the model must never delete someone else's file.
pub fn managed(app_data_dir: &Path) -> Option<PathBuf> {
    let path = model_dir(app_data_dir).join(MODEL_FILE);
    path.is_file().then_some(path)
}

/// Delete the downloaded model, along with any half-finished download.
///
/// Returns the number of bytes freed, so the UI can say what it recovered.
pub fn remove(app_data_dir: &Path) -> Result<u64> {
    let dir = model_dir(app_data_dir);
    let mut freed = 0;

    for name in [MODEL_FILE.to_string(), format!("{}.part", MODEL_FILE)] {
        let path = dir.join(name);
        if !path.is_file() {
            continue;
        }

        freed += std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        std::fs::remove_file(&path)
            .map_err(|e| VeloError::Storage(format!("Could not remove the model: {}", e)))?;
    }

    info!("removed whisper model, freed {} bytes", freed);
    Ok(freed)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_app_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("velo-model-test-{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(model_dir(&dir)).expect("could not create test dir");
        dir
    }

    #[test]
    fn remove_deletes_the_model_and_any_partial_download() {
        let dir = temp_app_dir("remove");
        std::fs::write(model_dir(&dir).join(MODEL_FILE), b"0123456789").unwrap();
        std::fs::write(
            model_dir(&dir).join(format!("{}.part", MODEL_FILE)),
            b"01234",
        )
        .unwrap();

        let freed = remove(&dir).expect("remove failed");
        assert_eq!(freed, 15, "should report the bytes it actually freed");
        assert!(managed(&dir).is_none(), "the model is still there");
        assert!(
            std::fs::read_dir(model_dir(&dir)).unwrap().next().is_none(),
            "a leftover file was not cleaned up"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_is_harmless_when_there_is_nothing_to_remove() {
        let dir = temp_app_dir("empty");
        assert_eq!(remove(&dir).expect("remove failed"), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn managed_ignores_files_outside_the_app_directory() {
        // `resolve` honours VELO_WHISPER_MODEL, but `managed` must not: it
        // decides what the app is allowed to delete.
        let dir = temp_app_dir("managed");
        let elsewhere = std::env::temp_dir().join("velo-someone-elses-model.bin");
        std::fs::write(&elsewhere, b"not ours").unwrap();

        assert!(managed(&dir).is_none());
        assert!(elsewhere.is_file(), "an unrelated file must be left alone");

        let _ = std::fs::remove_file(&elsewhere);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
