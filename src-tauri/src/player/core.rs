//! Safe encapsulation of the libmpv C handle.

use crate::errors::{Result, VeloError};
use crate::player::ffi::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;

pub struct MpvCore {
    handle: *mut mpv_handle,
}

// mpv_handle is thread-safe for many calls as per libmpv documentation,
// but we isolate raw access and synchronization inside PlayerManager.
unsafe impl Send for MpvCore {}
unsafe impl Sync for MpvCore {}

impl MpvCore {
    pub fn new() -> Result<Self> {
        unsafe {
            let handle = mpv_create();
            if handle.is_null() {
                return Err(VeloError::Player("Failed to create mpv handle".into()));
            }

            // Configure default options before mpv_initialize
            Self::set_option_str(handle, "config", "no")?;
            Self::set_option_str(handle, "load-scripts", "no")?;
            Self::set_option_str(handle, "idle", "yes")?;
            // macOS: mpv's own window backend ignores `wid`, so the video is
            // drawn through the render API into a GL view the app owns.
            #[cfg(target_os = "macos")]
            Self::set_option_str(handle, "vo", "libmpv")?;
            #[cfg(not(target_os = "macos"))]
            Self::set_option_str(handle, "vo", "gpu-next")?;
            Self::set_option_str(handle, "hwdec", "auto-safe")?;
            Self::set_option_str(handle, "keep-open", "yes")?;
            Self::set_option_str(handle, "terminal", "no")?;

            // The webview owns all input; mpv must not grab keys or the cursor.
            for (name, value) in [
                ("input-default-bindings", "no"),
                ("input-vo-keyboard", "no"),
                ("input-cursor", "no"),
            ] {
                if let Err(e) = Self::set_option_str(handle, name, value) {
                    tracing::warn!("mpv option {} unavailable: {}", name, e);
                }
            }

            // NOTE: mpv_initialize() is deliberately deferred to
            // `initialize()` -- `wid` is an init-only option and has no effect
            // once the core is initialized.
            Ok(Self { handle })
        }
    }

    /// Finish mpv startup, embedding into the given native view when one is
    /// available. Must be called exactly once, before any playback command.
    pub fn initialize(&self, wid: Option<i64>) -> Result<()> {
        unsafe {
            if let Some(wid) = wid {
                Self::set_option_str(self.handle, "wid", &wid.to_string())?;
            }

            let err = mpv_initialize(self.handle);
            if err < 0 {
                return Err(VeloError::Player(format!(
                    "mpv_initialize failed: {}",
                    Self::error_str(err)
                )));
            }

            Ok(())
        }
    }

    pub fn raw_handle(&self) -> *mut mpv_handle {
        self.handle
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

    fn set_option_str(handle: *mut mpv_handle, name: &str, value: &str) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let c_val = CString::new(value).map_err(|e| VeloError::Player(e.to_string()))?;
            let err = mpv_set_option_string(handle, c_name.as_ptr(), c_val.as_ptr());
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to set option {}={}: {}",
                    name,
                    value,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn set_property_string(&self, name: &str, value: &str) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let c_val = CString::new(value).map_err(|e| VeloError::Player(e.to_string()))?;
            let err = mpv_set_property_string(self.handle, c_name.as_ptr(), c_val.as_ptr());
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to set property {}={}: {}",
                    name,
                    value,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn set_property_bool(&self, name: &str, value: bool) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let mut flag: c_int = if value { 1 } else { 0 };
            let err = mpv_set_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_FLAG,
                &mut flag as *mut _ as *mut c_void,
            );
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to set bool property {}: {}",
                    name,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn set_property_double(&self, name: &str, value: f64) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let mut val = value;
            let err = mpv_set_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_DOUBLE,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to set double property {}: {}",
                    name,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn set_property_int(&self, name: &str, value: i64) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let mut val = value;
            let err = mpv_set_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_INT64,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to set int property {}: {}",
                    name,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn get_property_string(&self, name: &str) -> Option<String> {
        unsafe {
            let c_name = CString::new(name).ok()?;
            let ptr = mpv_get_property_string(self.handle, c_name.as_ptr());
            if ptr.is_null() {
                None
            } else {
                let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
                mpv_free(ptr as *mut c_void);
                Some(s)
            }
        }
    }

    pub fn get_property_double(&self, name: &str) -> Option<f64> {
        unsafe {
            let c_name = CString::new(name).ok()?;
            let mut val: f64 = 0.0;
            let err = mpv_get_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_DOUBLE,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_property_int(&self, name: &str) -> Option<i64> {
        unsafe {
            let c_name = CString::new(name).ok()?;
            let mut val: i64 = 0;
            let err = mpv_get_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_INT64,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 {
                None
            } else {
                Some(val)
            }
        }
    }

    pub fn get_property_bool(&self, name: &str) -> Option<bool> {
        unsafe {
            let c_name = CString::new(name).ok()?;
            let mut val: c_int = 0;
            let err = mpv_get_property(
                self.handle,
                c_name.as_ptr(),
                mpv_format::MPV_FORMAT_FLAG,
                &mut val as *mut _ as *mut c_void,
            );
            if err < 0 {
                None
            } else {
                Some(val != 0)
            }
        }
    }

    pub fn command(&self, args: &[&str]) -> Result<()> {
        unsafe {
            let c_strings: Vec<CString> = args
                .iter()
                .map(|&s| CString::new(s).map_err(|e| VeloError::Player(e.to_string())))
                .collect::<Result<Vec<_>>>()?;

            let mut c_ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
            c_ptrs.push(ptr::null());

            let err = mpv_command(self.handle, c_ptrs.as_mut_ptr());
            if err < 0 {
                Err(VeloError::Player(format!(
                    "mpv command {:?} failed: {}",
                    args,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }

    pub fn observe_property(&self, id: u64, name: &str, format: mpv_format) -> Result<()> {
        unsafe {
            let c_name = CString::new(name).map_err(|e| VeloError::Player(e.to_string()))?;
            let err = mpv_observe_property(self.handle, id, c_name.as_ptr(), format);
            if err < 0 {
                Err(VeloError::Player(format!(
                    "Failed to observe {}: {}",
                    name,
                    Self::error_str(err)
                )))
            } else {
                Ok(())
            }
        }
    }
}

impl Drop for MpvCore {
    fn drop(&mut self) {
        unsafe {
            if !self.handle.is_null() {
                mpv_terminate_destroy(self.handle);
                self.handle = ptr::null_mut();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpv_init() {
        let core = MpvCore::new();
        assert!(core.is_ok());
    }
}
