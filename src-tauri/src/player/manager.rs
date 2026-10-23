//! High-level PlayerManager coordinating mpv lifecycle, commands, and platform surfaces.

use crate::errors::Result;
use crate::platform::Platform;
use crate::player::core::MpvCore;
use crate::player::events::EventLoop;
use crate::player::types::PlaybackState;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::AppHandle;
use tracing::info;

pub struct PlayerManager {
    core: Arc<MpvCore>,
    _event_loop: Mutex<Option<EventLoop>>,
    state: Mutex<PlaybackState>,
    platform: Mutex<Platform>,
}

impl PlayerManager {
    pub fn new() -> Result<Self> {
        let core = Arc::new(MpvCore::new()?);
        Ok(Self {
            core,
            _event_loop: Mutex::new(None),
            state: Mutex::new(PlaybackState::Idle),
            platform: Mutex::new(Platform::new()),
        })
    }

    /// Finishes mpv startup. `wid` is the native window handle on platforms
    /// where mpv can embed itself; macOS passes `None` and renders through the
    /// render API instead.
    pub fn initialize(&self, app_handle: AppHandle, wid: Option<i64>) -> Result<()> {
        info!(
            "Initializing PlayerManager with native window wid={:?}",
            wid
        );
        self.core.initialize(wid)?;

        let event_loop = EventLoop::start(self.core.clone(), app_handle);
        *self._event_loop.lock() = Some(event_loop);

        Ok(())
    }

    /// The raw mpv handle, for binding a render context to this player.
    pub fn raw_handle(&self) -> *mut crate::player::ffi::mpv_handle {
        self.core.raw_handle()
    }

    pub fn current_path(&self) -> Option<String> {
        self.core.get_property_string("path")
    }

    /// True while mpv sits idle with nothing loaded. Playback commands are
    /// no-ops then: flipping `pause` on an empty core still emits a property
    /// change, which would announce a playback state the app is not in and
    /// leave the UI believing a file is open.
    fn is_idle(&self) -> bool {
        self.core.get_property_bool("idle-active").unwrap_or(false)
    }

    pub fn load_file(&self, path: &str, start_time: Option<f64>) -> Result<()> {
        info!("Loading file: {} (resume at {:?})", path, start_time);
        *self.state.lock() = PlaybackState::Loading;

        self.platform.lock().prevent_sleep(false);
        super::loading::load_paused(&self.core, path, start_time)
    }

    pub fn play(&self) -> Result<()> {
        if self.is_idle() {
            return Ok(());
        }
        self.core.set_property_bool("pause", false)?;
        self.platform.lock().prevent_sleep(true);
        *self.state.lock() = PlaybackState::Playing;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        if self.is_idle() {
            return Ok(());
        }
        self.core.set_property_bool("pause", true)?;
        self.platform.lock().prevent_sleep(false);
        *self.state.lock() = PlaybackState::Paused;
        Ok(())
    }

    pub fn toggle_pause(&self) -> Result<()> {
        if self.is_idle() {
            return Ok(());
        }
        let is_paused = self.core.get_property_bool("pause").unwrap_or(false);
        if is_paused {
            self.play()
        } else {
            self.pause()
        }
    }

    pub fn stop(&self) -> Result<()> {
        self.core.command(&["stop"])?;
        self.platform.lock().prevent_sleep(false);
        *self.state.lock() = PlaybackState::Stopped;
        Ok(())
    }

    pub fn seek(&self, seconds: f64, exact: bool) -> Result<()> {
        if self.is_idle() {
            return Ok(());
        }
        let flag = if exact { "exact" } else { "relative" };
        let sec_str = seconds.to_string();
        self.core.command(&["seek", &sec_str, flag])
    }

    pub fn seek_absolute(&self, seconds: f64) -> Result<()> {
        if self.is_idle() {
            return Ok(());
        }
        let sec_str = seconds.to_string();
        self.core.command(&["seek", &sec_str, "absolute+exact"])
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let clamped = volume.clamp(0.0, 150.0);
        self.core.set_property_double("volume", clamped)
    }

    pub fn set_mute(&self, muted: bool) -> Result<()> {
        self.core.set_property_bool("mute", muted)
    }

    pub fn set_speed(&self, speed: f64) -> Result<()> {
        let clamped = speed.clamp(0.25, 4.0);
        self.core.set_property_double("speed", clamped)
    }

    pub fn set_aspect_ratio(&self, ratio: &str) -> Result<()> {
        self.core
            .set_property_string("video-aspect-override", ratio)
    }

    pub fn select_audio_track(&self, id: i64) -> Result<()> {
        if id == 0 {
            self.core.set_property_string("aid", "no")
        } else {
            self.core.set_property_int("aid", id)
        }
    }

    pub fn select_subtitle_track(&self, id: i64) -> Result<()> {
        if id == 0 {
            self.core.set_property_string("sid", "no")
        } else {
            self.core.set_property_int("sid", id)
        }
    }

    pub fn add_subtitle_file(&self, path: &str) -> Result<()> {
        self.core.command(&["sub-add", path, "select"])
    }

    pub fn set_subtitle_delay(&self, seconds: f64) -> Result<()> {
        self.core.set_property_double("sub-delay", seconds)
    }

    pub fn set_audio_delay(&self, seconds: f64) -> Result<()> {
        self.core.set_property_double("audio-delay", seconds)
    }
}

impl Drop for PlayerManager {
    fn drop(&mut self) {
        // The render context holds GL resources tied to the mpv handle and
        // must be released before mpv is torn down.
        #[cfg(target_os = "macos")]
        unsafe {
            crate::player::render::destroy()
        };
    }
}

#[cfg(test)]
mod manual_loading_tests {
    use super::*;
    use crate::player::ffi::{mpv_event_id, mpv_wait_event};
    use std::time::{Duration, Instant};

    fn fixture() -> Option<(PlayerManager, String)> {
        let path = std::env::var("VELO_TEST_MEDIA").ok()?;
        assert!(std::path::Path::new(&path).is_file());
        let manager = PlayerManager::new().unwrap();
        manager.core.initialize(None).unwrap();
        manager.core.set_property_string("vo", "null").unwrap();
        manager.core.set_property_double("volume", 0.0).unwrap();
        Some((manager, path))
    }

    fn loaded(manager: &PlayerManager) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            unsafe {
                let event = mpv_wait_event(manager.core.raw_handle(), 0.1);
                if !event.is_null() && (*event).event_id == mpv_event_id::MPV_EVENT_FILE_LOADED {
                    return;
                }
            }
        }
        panic!("mpv did not load the fixture within 20 seconds");
    }

    fn assert_paused(manager: &PlayerManager) {
        assert_eq!(manager.core.get_property_bool("pause"), Some(true));
        assert!(matches!(
            super::super::events::loaded_state(&manager.core),
            PlaybackState::Paused
        ));
        let before = manager.core.get_property_double("time-pos").unwrap();
        std::thread::sleep(Duration::from_millis(250));
        let after = manager.core.get_property_double("time-pos").unwrap();
        assert!(
            (after - before).abs() < 0.05,
            "paused position advanced: {before} -> {after}"
        );
    }

    #[test]
    fn newly_loaded_file_waits_for_play() {
        let Some((manager, path)) = fixture() else {
            return;
        };
        manager.load_file(&path, None).unwrap();
        loaded(&manager);
        assert_paused(&manager);
    }

    #[test]
    fn replacing_a_playing_file_loads_paused() {
        let Some((manager, path)) = fixture() else {
            return;
        };
        manager.load_file(&path, None).unwrap();
        loaded(&manager);
        manager.play().unwrap();
        assert_eq!(manager.core.get_property_bool("pause"), Some(false));
        assert_eq!(
            super::super::events::loaded_state(&manager.core),
            PlaybackState::Playing
        );
        manager.load_file(&path, Some(30.0)).unwrap();
        loaded(&manager);
        assert_paused(&manager);
    }

    #[test]
    fn resume_waits_for_explicit_play_and_seeking_does_not_start_it() {
        let Some((manager, path)) = fixture() else {
            return;
        };
        manager.load_file(&path, Some(30.0)).unwrap();
        loaded(&manager);
        assert_paused(&manager);
        assert!(manager.core.get_property_double("time-pos").unwrap() >= 29.0);
        manager.seek_absolute(60.0).unwrap();
        // A seek moves the clock asynchronously even while paused. Wait for
        // that requested jump before measuring whether playback advances.
        let deadline = Instant::now() + Duration::from_secs(5);
        while (manager.core.get_property_double("time-pos").unwrap_or(0.0) - 60.0).abs() > 0.1 {
            assert!(Instant::now() < deadline, "seek did not reach its target");
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_paused(&manager);
        manager.play().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if manager.core.get_property_double("time-pos").unwrap_or(0.0) > 60.1 {
                manager.pause().unwrap();
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("explicit Play did not advance the recording");
    }
}
