//! Windows platform-specific HWND child window and power management.

use crate::errors::{Result, VeloError};
use std::os::raw::c_void;

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

pub struct WindowsPlatform;

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    pub unsafe fn setup_video_surface(parent_hwnd: *mut c_void) -> Result<i64> {
        if parent_hwnd.is_null() {
            return Err(VeloError::Platform("Parent HWND is null".into()));
        }

        // On Windows, the main HWND or a child HWND can be used directly as wid.
        Ok(parent_hwnd as i64)
    }

    #[cfg(not(target_os = "windows"))]
    pub unsafe fn setup_video_surface(_parent_hwnd: *mut c_void) -> Result<i64> {
        Err(VeloError::Platform("Windows platform not supported on this OS".into()))
    }

    pub fn prevent_sleep(&mut self, _prevent: bool) {
        #[cfg(target_os = "windows")]
        unsafe {
            if _prevent {
                SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED | ES_SYSTEM_REQUIRED);
            } else {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    }
}
