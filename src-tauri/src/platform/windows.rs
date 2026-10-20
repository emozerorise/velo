//! Windows platform-specific HWND child window and power management.

use crate::errors::{Result, VeloError};
use crate::player::viewport::{win32_rect, ClientMetrics, ResolvedViewport, SurfaceLayout};
use parking_lot::Mutex;
use std::os::raw::c_void;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetClientRect, RegisterClassW, SetWindowPos, CS_OWNDC,
    HTTRANSPARENT, HWND_BOTTOM, SWP_NOACTIVATE, WM_ERASEBKGND, WM_NCDESTROY, WM_NCHITTEST,
    WNDCLASSW, WS_CHILD, WS_CLIPSIBLINGS, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT, WS_VISIBLE,
};

static VIDEO_HWND: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());
static VIDEO_LAYOUT: Mutex<SurfaceLayout> = Mutex::new(SurfaceLayout::new());

unsafe extern "system" fn video_window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCHITTEST => HTTRANSPARENT as LRESULT,
        WM_ERASEBKGND => 1,
        WM_NCDESTROY => {
            let _ = VIDEO_HWND.compare_exchange(
                window,
                ptr::null_mut(),
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            DefWindowProcW(window, message, wparam, lparam)
        }
        _ => DefWindowProcW(window, message, wparam, lparam),
    }
}

fn native_error(action: &str) -> VeloError {
    VeloError::Platform(format!("{action}: {}", std::io::Error::last_os_error()))
}

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED,
};

pub struct WindowsPlatform;

impl Default for WindowsPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsPlatform {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "windows")]
    ///
    /// # Safety
    ///
    /// `parent_hwnd` must be live on this UI thread and outlive mpv. Windows
    /// destroys this owned child with its parent; toggling tools never does.
    pub unsafe fn setup_video_surface(parent_hwnd: *mut c_void) -> Result<i64> {
        if parent_hwnd.is_null() {
            return Err(VeloError::Platform("Parent HWND is null".into()));
        }

        let module = GetModuleHandleW(ptr::null());
        if module.is_null() {
            return Err(native_error("GetModuleHandleW"));
        }
        let name: Vec<u16> = "VeloVideoSurface\0".encode_utf16().collect();
        let class = WNDCLASSW {
            style: CS_OWNDC,
            lpfnWndProc: Some(video_window_proc),
            hInstance: module,
            lpszClassName: name.as_ptr(),
            ..std::mem::zeroed()
        };
        if RegisterClassW(&class) == 0 {
            return Err(native_error("RegisterClassW"));
        }
        let mut bounds: RECT = std::mem::zeroed();
        if GetClientRect(parent_hwnd, &mut bounds) == 0 {
            return Err(native_error("GetClientRect"));
        }
        let child = CreateWindowExW(
            WS_EX_TRANSPARENT | WS_EX_NOACTIVATE,
            name.as_ptr(),
            ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS,
            0,
            0,
            (bounds.right - bounds.left).max(1),
            (bounds.bottom - bounds.top).max(1),
            parent_hwnd,
            ptr::null_mut(),
            module,
            ptr::null(),
        );
        if child.is_null() {
            return Err(native_error("CreateWindowExW"));
        }
        VIDEO_HWND.store(child, Ordering::Release);
        if SetWindowPos(
            child,
            HWND_BOTTOM,
            0,
            0,
            bounds.right.max(1),
            bounds.bottom.max(1),
            SWP_NOACTIVATE,
        ) == 0
        {
            return Err(native_error("SetWindowPos"));
        }
        Ok(child as i64)
    }

    /// # Safety
    /// The parent and its video child must be live on the current UI thread.
    pub unsafe fn apply_video_viewport(
        _parent_hwnd: *mut c_void,
        viewport: &ResolvedViewport,
        client: ClientMetrics,
    ) -> Result<()> {
        if let Some(rect) = VIDEO_LAYOUT.lock().apply(*viewport, client) {
            Self::set_video_frame(rect, client.scale_factor)?;
        }
        Ok(())
    }

    /// # Safety
    /// `parent_hwnd` must be live on this UI thread; scale is Tauri's DPI factor.
    pub unsafe fn sync_video_surface(parent_hwnd: *mut c_void, scale: f64) -> Result<()> {
        let mut bounds: RECT = std::mem::zeroed();
        if GetClientRect(parent_hwnd, &mut bounds) == 0 {
            return Err(native_error("GetClientRect"));
        }
        let client = ClientMetrics {
            width: (bounds.right - bounds.left) as f64 / scale,
            height: (bounds.bottom - bounds.top) as f64 / scale,
            scale_factor: scale,
        };
        let rect = VIDEO_LAYOUT.lock().resize(client);
        Self::set_video_frame(rect, scale)
    }

    unsafe fn set_video_frame(rect: crate::player::viewport::Rect, scale: f64) -> Result<()> {
        let child = VIDEO_HWND.load(Ordering::Acquire);
        if child.is_null() {
            return Err(VeloError::Platform(
                "Video child window is unavailable".into(),
            ));
        }
        let frame = win32_rect(rect, scale);
        if SetWindowPos(
            child,
            HWND_BOTTOM,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            SWP_NOACTIVATE,
        ) == 0
        {
            return Err(native_error("SetWindowPos"));
        }
        Ok(())
    }

    ///
    /// # Safety
    ///
    /// Always fails on non-Windows targets; the argument is ignored.
    #[cfg(not(target_os = "windows"))]
    pub unsafe fn setup_video_surface(_parent_hwnd: *mut c_void) -> Result<i64> {
        Err(VeloError::Platform(
            "Windows platform not supported on this OS".into(),
        ))
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
