//! Pins down the `idle-active` property that the playback-command guards in
//! `PlayerManager` rely on. If mpv ever reported it the other way round,
//! play/pause would silently become no-ops during playback.
//!
//! Skips itself unless `VELO_TEST_MEDIA` points at a real media file.

use std::path::PathBuf;
use velo_lib::player::core::MpvCore;
use velo_lib::player::ffi::{mpv_event_id, mpv_wait_event};

fn sample_media() -> Option<String> {
    let path = std::env::var("VELO_TEST_MEDIA").ok()?;
    PathBuf::from(&path).is_file().then_some(path)
}

#[test]
fn idle_active_reports_whether_a_file_is_loaded() {
    let Some(media) = sample_media() else {
        eprintln!("skipping: set VELO_TEST_MEDIA to a media file");
        return;
    };

    let core = MpvCore::new().expect("mpv handle");
    core.initialize(None).expect("mpv init");
    let _ = core.set_property_double("volume", 0.0);

    assert_eq!(
        core.get_property_bool("idle-active"),
        Some(true),
        "a freshly initialized core with no file should be idle"
    );

    core.command(&["loadfile", &media, "replace"])
        .expect("loadfile");

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    let mut loaded = false;
    while !loaded && std::time::Instant::now() < deadline {
        unsafe {
            let event_ptr = mpv_wait_event(core.raw_handle(), 0.25);
            if event_ptr.is_null() {
                continue;
            }
            if (*event_ptr).event_id == mpv_event_id::MPV_EVENT_FILE_LOADED {
                loaded = true;
            }
        }
    }
    assert!(loaded, "mpv never reported the file as loaded");

    assert_eq!(
        core.get_property_bool("idle-active"),
        Some(false),
        "a core with a file loaded must not look idle, or playback commands \
         would be skipped"
    );
}
