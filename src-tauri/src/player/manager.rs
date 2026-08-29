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

    pub fn load_file(&self, path: &str, start_time: Option<f64>) -> Result<()> {
        info!("Loading file: {}", path);
        *self.state.lock() = PlaybackState::Loading;

        if let Some(pos) = start_time {
            let start_arg = format!("start={}", pos);
            self.core
                .command(&["loadfile", path, "replace", &start_arg])?;
        } else {
            self.core.command(&["loadfile", path, "replace"])?;
        }

        self.platform.lock().prevent_sleep(true);
        Ok(())
    }

    pub fn play(&self) -> Result<()> {
        self.core.set_property_bool("pause", false)?;
        self.platform.lock().prevent_sleep(true);
        *self.state.lock() = PlaybackState::Playing;
        Ok(())
    }

    pub fn pause(&self) -> Result<()> {
        self.core.set_property_bool("pause", true)?;
        self.platform.lock().prevent_sleep(false);
        *self.state.lock() = PlaybackState::Paused;
        Ok(())
    }

    pub fn toggle_pause(&self) -> Result<()> {
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
        let flag = if exact { "exact" } else { "relative" };
        let sec_str = seconds.to_string();
        self.core.command(&["seek", &sec_str, flag])
    }

    pub fn seek_absolute(&self, seconds: f64) -> Result<()> {
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
