//! libmpv render API: draws video into an OpenGL surface the app owns,
//! instead of letting mpv open a window of its own.
//!
//! The context is process-global because the draw call originates from a
//! native view's `drawRect:`, which has no room to carry Rust state.

use crate::errors::{Result, VeloError};
use crate::player::ffi::*;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

static RENDER_CONTEXT: AtomicPtr<mpv_render_context> = AtomicPtr::new(ptr::null_mut());

/// Resolves GL entry points for mpv. The context must already be current.
unsafe extern "C" fn get_proc_address(_ctx: *mut c_void, name: *const c_char) -> *mut c_void {
    static OPENGL_FRAMEWORK: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

    let mut handle = OPENGL_FRAMEWORK.load(Ordering::Acquire);
    if handle.is_null() {
        let path = match CString::new("/System/Library/Frameworks/OpenGL.framework/OpenGL") {
            Ok(p) => p,
            Err(_) => return ptr::null_mut(),
        };
        handle = libc_dlopen(path.as_ptr(), RTLD_LAZY | RTLD_LOCAL);
        if handle.is_null() {
            return ptr::null_mut();
        }
        OPENGL_FRAMEWORK.store(handle, Ordering::Release);
    }

    libc_dlsym(handle, name)
}

const RTLD_LAZY: c_int = 0x1;
const RTLD_LOCAL: c_int = 0x4;

extern "C" {
    #[link_name = "dlopen"]
    fn libc_dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    #[link_name = "dlsym"]
    fn libc_dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

/// Creates the render context. The caller's GL context must be current on the
/// calling thread, and `mpv` must already be initialized.
pub unsafe fn create(mpv: *mut mpv_handle) -> Result<()> {
    if !RENDER_CONTEXT.load(Ordering::Acquire).is_null() {
        return Ok(());
    }

    let mut init_params = mpv_opengl_init_params {
        get_proc_address: Some(get_proc_address),
        get_proc_address_ctx: ptr::null_mut(),
    };
    let mut params = [
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_API_TYPE,
            data: MPV_RENDER_API_TYPE_OPENGL.as_ptr() as *mut c_void,
        },
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_OPENGL_INIT_PARAMS,
            data: &mut init_params as *mut _ as *mut c_void,
        },
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_INVALID,
            data: ptr::null_mut(),
        },
    ];

    let mut ctx: *mut mpv_render_context = ptr::null_mut();
    let err = mpv_render_context_create(&mut ctx, mpv, params.as_mut_ptr());
    if err < 0 || ctx.is_null() {
        return Err(VeloError::Player(format!(
            "mpv_render_context_create failed ({})",
            err
        )));
    }

    RENDER_CONTEXT.store(ctx, Ordering::Release);
    Ok(())
}

/// Registers the callback mpv fires (from an arbitrary thread) when a new
/// frame is ready to be drawn.
pub unsafe fn set_update_callback(callback: unsafe extern "C" fn(*mut c_void)) {
    let ctx = RENDER_CONTEXT.load(Ordering::Acquire);
    if !ctx.is_null() {
        mpv_render_context_set_update_callback(ctx, Some(callback), ptr::null_mut());
    }
}

/// Draws the current frame into `fbo` at the given pixel size. Must run with
/// the target GL context current.
pub unsafe fn render(fbo: c_int, width: c_int, height: c_int) -> bool {
    let ctx = RENDER_CONTEXT.load(Ordering::Acquire);
    if ctx.is_null() {
        return false;
    }

    let mut fbo_param = mpv_opengl_fbo {
        fbo,
        w: width,
        h: height,
        internal_format: 0,
    };
    // AppKit's GL surfaces are already bottom-up relative to mpv's output.
    let mut flip_y: c_int = 1;

    let mut params = [
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_OPENGL_FBO,
            data: &mut fbo_param as *mut _ as *mut c_void,
        },
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_FLIP_Y,
            data: &mut flip_y as *mut _ as *mut c_void,
        },
        mpv_render_param {
            type_: mpv_render_param_type::MPV_RENDER_PARAM_INVALID,
            data: ptr::null_mut(),
        },
    ];

    mpv_render_context_render(ctx, params.as_mut_ptr()) >= 0
}

/// Tells mpv the frame reached the screen, so it can pace playback.
pub unsafe fn report_swap() {
    let ctx = RENDER_CONTEXT.load(Ordering::Acquire);
    if !ctx.is_null() {
        mpv_render_context_report_swap(ctx);
    }
}

/// Must be called before the mpv handle is destroyed.
pub unsafe fn destroy() {
    let ctx = RENDER_CONTEXT.swap(ptr::null_mut(), Ordering::AcqRel);
    if !ctx.is_null() {
        mpv_render_context_free(ctx);
    }
}
