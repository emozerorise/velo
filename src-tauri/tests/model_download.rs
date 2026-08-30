//! Checks the model download against the real host.
//!
//! Skipped unless `VELO_TEST_NETWORK=1`, so neither CI nor a normal
//! `cargo test` reaches out to the network -- or pulls half a gigabyte:
//!
//! ```sh
//! VELO_TEST_NETWORK=1 cargo test --test model_download -- --nocapture
//! ```

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use velo_lib::transcript::model;

fn network_enabled() -> bool {
    std::env::var("VELO_TEST_NETWORK").as_deref() == Ok("1")
}

/// Cancels after the first megabyte: enough to prove the URL resolves, the
/// stream flows and progress is reported, without fetching the whole model.
#[tokio::test]
async fn download_streams_then_cancels_cleanly() {
    if !network_enabled() {
        eprintln!("skipping: set VELO_TEST_NETWORK=1 to hit the network");
        return;
    }

    let dir = std::env::temp_dir().join("velo-model-download-test");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("could not create temp dir");

    let cancel = Arc::new(AtomicBool::new(false));
    let received = Arc::new(AtomicU64::new(0));
    let total = Arc::new(AtomicU64::new(0));

    let stop = cancel.clone();
    let seen = received.clone();
    let seen_total = total.clone();

    let result = model::download(&dir, cancel.clone(), move |got, size| {
        seen.store(got, Ordering::Relaxed);
        seen_total.store(size, Ordering::Relaxed);
        if got > 1024 * 1024 {
            stop.store(true, Ordering::Relaxed);
        }
    })
    .await;

    assert!(result.is_err(), "a cancelled download should not succeed");
    assert!(
        received.load(Ordering::Relaxed) > 0,
        "no bytes arrived, so the URL or the stream is wrong"
    );
    assert_eq!(
        total.load(Ordering::Relaxed),
        model::MODEL_BYTES,
        "the host reports a different size than MODEL_BYTES claims"
    );

    let model_dir = model::model_dir(&dir);
    let leftovers: Vec<_> = std::fs::read_dir(&model_dir)
        .expect("model dir missing")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        leftovers.is_empty(),
        "a cancelled download left files behind: {:?}",
        leftovers
    );

    // Nothing was completed, so nothing should resolve as a usable model.
    assert!(model::resolve(&dir).is_none() || std::env::var("VELO_WHISPER_MODEL").is_ok());

    let _ = std::fs::remove_dir_all(&dir);
}
