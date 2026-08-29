//! Background event listener thread translating libmpv C events into Tauri events.

use crate::player::core::MpvCore;
use crate::player::ffi::*;
use crate::player::types::{MediaInfo, PlaybackState, TimeUpdate, Track};
use std::ffi::CStr;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tracing::{debug, info};

pub const OBSERVE_PAUSE: u64 = 1;
pub const OBSERVE_TIME_POS: u64 = 2;
pub const OBSERVE_DURATION: u64 = 3;
pub const OBSERVE_VOLUME: u64 = 4;
pub const OBSERVE_MUTE: u64 = 5;
pub const OBSERVE_SPEED: u64 = 6;
pub const OBSERVE_EOF: u64 = 7;

pub struct EventLoop {
    running: Arc<AtomicBool>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn start(core: Arc<MpvCore>, app_handle: AppHandle) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        // Setup property observations
        let _ = core.observe_property(OBSERVE_PAUSE, "pause", mpv_format::MPV_FORMAT_FLAG);
        let _ = core.observe_property(OBSERVE_TIME_POS, "time-pos", mpv_format::MPV_FORMAT_DOUBLE);
        let _ = core.observe_property(OBSERVE_DURATION, "duration", mpv_format::MPV_FORMAT_DOUBLE);
        let _ = core.observe_property(OBSERVE_VOLUME, "volume", mpv_format::MPV_FORMAT_DOUBLE);
        let _ = core.observe_property(OBSERVE_MUTE, "mute", mpv_format::MPV_FORMAT_FLAG);
        let _ = core.observe_property(OBSERVE_SPEED, "speed", mpv_format::MPV_FORMAT_DOUBLE);
        let _ = core.observe_property(OBSERVE_EOF, "eof-reached", mpv_format::MPV_FORMAT_FLAG);

        std::thread::Builder::new()
            .name("velo-mpv-event-loop".into())
            .spawn(move || {
                info!("mpv event loop started");
                let mut last_time_pos_emit = Instant::now();
                let time_pos_interval = Duration::from_millis(100); // 10Hz throttle

                let mut current_duration: f64 = 0.0;

                while running_clone.load(Ordering::Relaxed) {
                    unsafe {
                        let event_ptr = mpv_wait_event(core.raw_handle(), 0.25);
                        if event_ptr.is_null() {
                            continue;
                        }

                        let event = &*event_ptr;
                        match event.event_id {
                            mpv_event_id::MPV_EVENT_NONE => {}
                            mpv_event_id::MPV_EVENT_SHUTDOWN => {
                                info!("mpv shutdown received");
                                break;
                            }
                            mpv_event_id::MPV_EVENT_FILE_LOADED => {
                                info!("mpv file loaded");
                                if let Some(dur) = core.get_property_double("duration") {
                                    current_duration = dur;
                                }

                                let media_info = MediaInfo {
                                    path: core.get_property_string("path").unwrap_or_default(),
                                    file_name: core.get_property_string("filename").unwrap_or_default(),
                                    duration: current_duration,
                                    width: core.get_property_int("width").unwrap_or(0),
                                    height: core.get_property_int("height").unwrap_or(0),
                                    video_codec: core.get_property_string("video-codec"),
                                    audio_codec: core.get_property_string("audio-codec"),
                                    fps: core.get_property_double("container-fps").unwrap_or(0.0),
                                    hwdec_current: core.get_property_string("hwdec-current"),
                                };

                                let _ = app_handle.emit("velo://media-loaded", &media_info);
                                let _ = app_handle.emit("velo://player-state", PlaybackState::Playing);
                                
                                // Fetch tracks
                                Self::emit_tracks(&core, &app_handle);
                            }
                            mpv_event_id::MPV_EVENT_END_FILE => {
                                debug!("mpv end file");
                                let _ = app_handle.emit("velo://player-state", PlaybackState::Ended);
                                let _ = app_handle.emit("velo://playback-ended", ());
                            }
                            mpv_event_id::MPV_EVENT_PROPERTY_CHANGE => {
                                if !event.data.is_null() {
                                    let prop = &*(event.data as *const mpv_event_property);
                                    let prop_name = if !prop.name.is_null() {
                                        CStr::from_ptr(prop.name).to_string_lossy()
                                    } else {
                                        continue;
                                    };

                                    match prop_name.as_ref() {
                                        "pause" => {
                                            if prop.format == mpv_format::MPV_FORMAT_FLAG {
                                                let is_paused = *(prop.data as *const c_int) != 0;
                                                let state = if is_paused {
                                                    PlaybackState::Paused
                                                } else {
                                                    PlaybackState::Playing
                                                };
                                                let _ = app_handle.emit("velo://player-state", state);
                                            }
                                        }
                                        "time-pos" => {
                                            if prop.format == mpv_format::MPV_FORMAT_DOUBLE {
                                                let now = Instant::now();
                                                if now.duration_since(last_time_pos_emit) >= time_pos_interval {
                                                    last_time_pos_emit = now;
                                                    let current_time = *(prop.data as *const f64);
                                                    let percent = if current_duration > 0.0 {
                                                        (current_time / current_duration) * 100.0
                                                    } else {
                                                        0.0
                                                    };

                                                    let update = TimeUpdate {
                                                        current_time,
                                                        duration: current_duration,
                                                        percent,
                                                    };
                                                    let _ = app_handle.emit("velo://time-update", update);
                                                }
                                            }
                                        }
                                        "duration" => {
                                            if prop.format == mpv_format::MPV_FORMAT_DOUBLE {
                                                current_duration = *(prop.data as *const f64);
                                            }
                                        }
                                        "volume" => {
                                            if prop.format == mpv_format::MPV_FORMAT_DOUBLE {
                                                let vol = *(prop.data as *const f64);
                                                let _ = app_handle.emit("velo://volume-changed", vol);
                                            }
                                        }
                                        "mute" => {
                                            if prop.format == mpv_format::MPV_FORMAT_FLAG {
                                                let is_muted = *(prop.data as *const c_int) != 0;
                                                let _ = app_handle.emit("velo://mute-changed", is_muted);
                                            }
                                        }
                                        "speed" => {
                                            if prop.format == mpv_format::MPV_FORMAT_DOUBLE {
                                                let spd = *(prop.data as *const f64);
                                                let _ = app_handle.emit("velo://speed-changed", spd);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            mpv_event_id::MPV_EVENT_TRACKS_CHANGED => {
                                Self::emit_tracks(&core, &app_handle);
                            }
                            mpv_event_id::MPV_EVENT_LOG_MESSAGE => {
                                if !event.data.is_null() {
                                    let msg = &*(event.data as *const mpv_event_log_message);
                                    let text = CStr::from_ptr(msg.text).to_string_lossy();
                                    debug!("mpv log: {}", text.trim());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                info!("mpv event loop exited");
            })
            .expect("Failed to spawn mpv event thread");

        Self { running }
    }

    fn emit_tracks(core: &MpvCore, app_handle: &AppHandle) {
        let count = core.get_property_int("track-list/count").unwrap_or(0);
        let mut tracks = Vec::new();

        for i in 0..count {
            let id = core
                .get_property_int(&format!("track-list/{}/id", i))
                .unwrap_or(0);
            let track_type = core
                .get_property_string(&format!("track-list/{}/type", i))
                .unwrap_or_default();
            let title = core.get_property_string(&format!("track-list/{}/title", i));
            let lang = core.get_property_string(&format!("track-list/{}/lang", i));
            let codec = core.get_property_string(&format!("track-list/{}/codec", i));
            let selected = core
                .get_property_bool(&format!("track-list/{}/selected", i))
                .unwrap_or(false);
            let default = core
                .get_property_bool(&format!("track-list/{}/default", i))
                .unwrap_or(false);

            tracks.push(Track {
                id,
                track_type,
                src_id: i,
                title,
                lang,
                codec,
                selected,
                default,
            });
        }

        let _ = app_handle.emit("velo://tracks-changed", tracks);
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
