//! Headless audio extraction through a second libmpv instance.
//!
//! mpv's encode mode is already linked into the app, so producing the 16 kHz
//! mono WAV whisper wants costs no extra dependency and no bundled ffmpeg.
//! The handle is entirely separate from the playback one, so extraction never
//! touches the video the user is watching.

use crate::errors::{Result, VeloError};
use crate::player::ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct AudioExtractor {
    handle: *mut mpv_handle,
}

const OBSERVE_TIME_POS: u64 = 1;

impl AudioExtractor {
    fn new(dest: &Path) -> Result<Self> {
        let dest = dest
            .to_str()
            .ok_or_else(|| VeloError::InvalidParameter("Non-UTF8 output path".into()))?;

        unsafe {
            let handle = mpv_create();
            if handle.is_null() {
                return Err(VeloError::Player("Failed to create mpv handle".into()));
            }
            let extractor = Self { handle };

            for (name, value) in [
                ("config", "no"),
                ("load-scripts", "no"),
                ("terminal", "no"),
                ("idle", "no"),
                // Nothing to show and nothing to play out loud: this instance
                // only walks the file as fast as it can decode it.
                ("vo", "null"),
                ("vid", "no"),
                ("sid", "no"),
                ("ao", "null"),
                // whisper wants 16 kHz mono PCM.
                ("af", "aresample=16000,aformat=channel_layouts=mono"),
                ("of", "wav"),
                ("oac", "pcm_s16le"),
                ("o", dest),
            ] {
                extractor.set_option(name, value)?;
            }

            let err = mpv_initialize(handle);
            if err < 0 {
                return Err(VeloError::Player(format!(
                    "mpv_initialize failed for extraction: {}",
                    error_str(err)
                )));
            }

            Ok(extractor)
        }
    }

    fn set_option(&self, name: &str, value: &str) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let c_val = CString::new(value).map_err(|e| VeloError::Player(e.to_string()))?;
            let err = mpv_set_option_string(self.handle, c_name.as_ptr(), c_val.as_ptr());
            if err < 0 {
                return Err(VeloError::Player(format!(
                    "Failed to set extraction option {}={}: {}",
                    name,
                    value,
                    error_str(err)
                )));
            }
            Ok(())
        }
    }

    fn command(&self, args: &[&str]) -> Result<()> {
        unsafe {
            let c_strings: Vec<CString> = args
                .iter()
                .map(|&s| CString::new(s).map_err(|e| VeloError::Player(e.to_string())))
                .collect::<Result<Vec<_>>>()?;
            let mut c_ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
            c_ptrs.push(ptr::null());

            let err = mpv_command(self.handle, c_ptrs.as_mut_ptr());
            if err < 0 {
                return Err(VeloError::Player(format!(
                    "mpv extraction command {:?} failed: {}",
                    args,
                    error_str(err)
                )));
            }
            Ok(())
        }
    }

    fn duration(&self) -> Option<f64> {
        unsafe {
            let c_name = CString::new("duration").ok()?;
            let mut val: f64 = 0.0;
            let err = mpv_get_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_DOUBLE,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 || val <= 0.0 {
                None
            } else {
                Some(val)
            }
        }
    }
}

impl Drop for AudioExtractor {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                mpv_terminate_destroy(self.handle);
                self.handle = ptr::null_mut();
            }
        }
    }
}

fn error_str(err: c_int) -> String {
    unsafe {
        let ptr = mpv_error_string(err);
        if ptr.is_null() {
            format!("Unknown error ({})", err)
        } else {
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}

/// Read back a WAV written by `extract` as the f32 samples whisper wants.
///
/// Deliberately narrow: it accepts only the 16 kHz mono 16-bit format this
/// module produces, so a mismatch surfaces here rather than as garbled
/// transcription later.
pub fn read_samples(path: &Path) -> Result<Vec<f32>> {
    let bytes = std::fs::read(path)
        .map_err(|e| VeloError::Storage(format!("Could not read the extracted audio: {}", e)))?;

    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(VeloError::Player("The extracted audio is not a WAV".into()));
    }

    let mut channels = 0_u16;
    let mut sample_rate = 0_u32;
    let mut bits = 0_u16;
    let mut data: Option<&[u8]> = None;
    let mut pos = 12;

    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = u32::from_le_bytes([
            bytes[pos + 4],
            bytes[pos + 5],
            bytes[pos + 6],
            bytes[pos + 7],
        ]) as usize;

        let body = pos + 8;
        let end = body.saturating_add(size).min(bytes.len());

        match id {
            b"fmt " if end >= body + 16 => {
                channels = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]);
                sample_rate = u32::from_le_bytes([
                    bytes[body + 4],
                    bytes[body + 5],
                    bytes[body + 6],
                    bytes[body + 7],
                ]);
                bits = u16::from_le_bytes([bytes[body + 14], bytes[body + 15]]);
            }
            b"data" => data = Some(&bytes[body..end]),
            _ => {}
        }

        // Chunks are word-aligned, so an odd size is followed by a pad byte.
        pos = body + size + (size & 1);
    }

    if channels != 1 || sample_rate != 16_000 || bits != 16 {
        return Err(VeloError::Player(format!(
            "Expected 16 kHz mono 16-bit audio, got {} Hz, {} channel(s), {}-bit",
            sample_rate, channels, bits
        )));
    }

    let data = data.ok_or_else(|| VeloError::Player("The extracted audio is empty".into()))?;
    let pcm: Vec<i16> = data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| i16::from_le_bytes(*pair))
        .collect();

    let mut samples = vec![0.0_f32; pcm.len()];
    whisper_rs::convert_integer_to_float_audio(&pcm, &mut samples)
        .map_err(|e| VeloError::Player(format!("Could not convert the audio: {}", e)))?;

    Ok(samples)
}

/// Decode the audio of `src` into a 16 kHz mono WAV at `dest`.
///
/// `on_progress` receives 0.0..1.0, or -1.0 while the duration is unknown.
pub fn extract(
    src: &str,
    dest: &Path,
    cancel: &Arc<AtomicBool>,
    on_progress: &dyn Fn(f64),
) -> Result<()> {
    if !Path::new(src).exists() {
        return Err(VeloError::FileNotFound(src.to_string()));
    }

    let extractor = AudioExtractor::new(dest)?;
    unsafe {
        let c_name = CString::new("time-pos").map_err(|e| VeloError::Player(e.to_string()))?;
        mpv_observe_property(
            extractor.handle,
            OBSERVE_TIME_POS,
            c_name.as_ptr(),
            mpv_format::MPV_FORMAT_DOUBLE,
        );
    }

    extractor.command(&["loadfile", src])?;
    on_progress(-1.0);

    let mut duration = 0.0_f64;
    let mut finished = false;

    while !finished {
        if cancel.load(Ordering::Relaxed) {
            let _ = extractor.command(&["quit"]);
            return Err(VeloError::Player("Transcription cancelled".into()));
        }

        unsafe {
            let event_ptr = mpv_wait_event(extractor.handle, 0.25);
            if event_ptr.is_null() {
                continue;
            }
            let event = &*event_ptr;

            match event.event_id {
                mpv_event_id::MPV_EVENT_FILE_LOADED => {
                    duration = extractor.duration().unwrap_or(0.0);
                }
                mpv_event_id::MPV_EVENT_PROPERTY_CHANGE => {
                    if event.reply_userdata == OBSERVE_TIME_POS && duration > 0.0 {
                        let prop = &*(event.data as *mut mpv_event_property);
                        if prop.format == mpv_format::MPV_FORMAT_DOUBLE && !prop.data.is_null() {
                            let pos = *(prop.data as *mut f64);
                            on_progress((pos / duration).clamp(0.0, 1.0));
                        }
                    }
                }
                mpv_event_id::MPV_EVENT_END_FILE => {
                    let end = &*(event.data as *mut mpv_event_end_file);
                    if end.error < 0 {
                        return Err(VeloError::Player(format!(
                            "Audio extraction failed: {}",
                            error_str(end.error)
                        )));
                    }
                    finished = true;
                }
                mpv_event_id::MPV_EVENT_SHUTDOWN => {
                    finished = true;
                }
                _ => {}
            }
        }
    }

    // Encode mode writes the WAV header/trailer as the core shuts down, so the
    // file is only complete once mpv has actually quit.
    let _ = extractor.command(&["quit"]);
    drop(extractor);

    let size = std::fs::metadata(dest).map(|m| m.len()).unwrap_or(0);
    if size < 1024 {
        return Err(VeloError::Player(
            "No audio was extracted -- the file may have no audio track".into(),
        ));
    }

    on_progress(1.0);
    Ok(())
}
