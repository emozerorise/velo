use crate::errors::{Result, VeloError};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSSize {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

/// NSView autoresizing: track the superview in both axes.
const NS_VIEW_WIDTH_SIZABLE: u64 = 2;
const NS_VIEW_HEIGHT_SIZABLE: u64 = 16;
/// NSWindowOrderingMode::NSWindowBelow
const NS_WINDOW_BELOW: i64 = -1;

// NSOpenGLPixelFormatAttribute values.
const NS_OPENGL_PFA_DOUBLE_BUFFER: u32 = 5;
const NS_OPENGL_PFA_ACCELERATED: u32 = 73;
const NS_OPENGL_PFA_ALLOW_OFFLINE: u32 = 96;
const NS_OPENGL_PFA_OPENGL_PROFILE: u32 = 99;
const NS_OPENGL_PROFILE_VERSION_3_2_CORE: u32 = 0x3200;

#[link(name = "objc", kind = "dylib")]
extern "C" {
    fn objc_getClass(name: *const c_char) -> *mut c_void;
    fn sel_registerName(name: *const c_char) -> *mut c_void;
    fn objc_allocateClassPair(
        superclass: *mut c_void,
        name: *const c_char,
        extra_bytes: usize,
    ) -> *mut c_void;
    fn objc_registerClassPair(cls: *mut c_void);
    fn class_addMethod(
        cls: *mut c_void,
        name: *mut c_void,
        imp: *const c_void,
        types: *const c_char,
    ) -> i8;
}

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: *const c_void,
        assertion_level: u32,
        assertion_name: *const c_void,
        assertion_id: *mut u32,
    ) -> i32;
    fn IOPMAssertionRelease(assertion_id: u32) -> i32;
}

/// The GL view mpv renders into, a subview of the window's content view.
static VIDEO_VIEW: AtomicPtr<c_void> = AtomicPtr::new(ptr::null_mut());

unsafe fn get_class(name: &[u8]) -> *mut c_void {
    objc_getClass(name.as_ptr() as *const c_char)
}

unsafe fn get_sel(name: &[u8]) -> *mut c_void {
    sel_registerName(name.as_ptr() as *const c_char)
}

unsafe fn msg_obj(receiver: *mut c_void, sel: *mut c_void) -> *mut c_void {
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    f(receiver, sel)
}

unsafe fn msg_void(receiver: *mut c_void, sel: *mut c_void) {
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void) =
        std::mem::transmute(objc_msgSend as *const ());
    f(receiver, sel)
}

unsafe fn msg_set_bool(receiver: *mut c_void, sel: *mut c_void, value: bool) {
    let f: unsafe extern "C" fn(*mut c_void, *mut c_void, i8) =
        std::mem::transmute(objc_msgSend as *const ());
    f(receiver, sel, value as i8)
}

/// NSRect is returned indirectly on aarch64 and via objc_msgSend_stret on x86_64.
unsafe fn rect_of(obj: *mut c_void, sel_name: &[u8]) -> NSRect {
    let sel = get_sel(sel_name);

    #[cfg(target_arch = "aarch64")]
    {
        let f: unsafe extern "C" fn(*mut c_void, *mut c_void) -> NSRect =
            std::mem::transmute(objc_msgSend as *const ());
        f(obj, sel)
    }

    #[cfg(target_arch = "x86_64")]
    {
        let mut out = NSRect::default();
        let f: unsafe extern "C" fn(*mut NSRect, *mut c_void, *mut c_void) =
            std::mem::transmute(objc_msgSend_stret as *const ());
        f(&mut out, obj, sel);
        out
    }
}

unsafe fn convert_rect_to_backing(view: *mut c_void, rect: NSRect) -> NSRect {
    let sel = get_sel(b"convertRectToBacking:\0");

    #[cfg(target_arch = "aarch64")]
    {
        let f: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) -> NSRect =
            std::mem::transmute(objc_msgSend as *const ());
        f(view, sel, rect)
    }

    #[cfg(target_arch = "x86_64")]
    {
        let mut out = NSRect::default();
        let f: unsafe extern "C" fn(*mut NSRect, *mut c_void, *mut c_void, NSRect) =
            std::mem::transmute(objc_msgSend_stret as *const ());
        f(&mut out, view, sel, rect);
        out
    }
}

/// `drawRect:` -- hands the view's GL surface to mpv.
extern "C" fn video_view_draw_rect(this: *mut c_void, _sel: *mut c_void, _dirty: NSRect) {
    unsafe {
        let gl_context = msg_obj(this, get_sel(b"openGLContext\0"));
        if gl_context.is_null() {
            return;
        }
        msg_void(gl_context, get_sel(b"makeCurrentContext\0"));

        let backing = convert_rect_to_backing(this, rect_of(this, b"bounds\0"));
        let width = backing.size.width.max(1.0) as c_int;
        let height = backing.size.height.max(1.0) as c_int;

        crate::player::render::render(0, width, height);

        msg_void(gl_context, get_sel(b"flushBuffer\0"));
        crate::player::render::report_swap();
    }
}

/// `veloRequestRedraw` -- marks the view dirty, always on the main thread.
extern "C" fn video_view_request_redraw(this: *mut c_void, _sel: *mut c_void) {
    unsafe { msg_set_bool(this, get_sel(b"setNeedsDisplay:\0"), true) }
}

/// `hitTest:` -- the video surface must never consume mouse events, or it
/// would swallow clicks meant for the webview UI layered above it.
extern "C" fn video_view_hit_test(
    _this: *mut c_void,
    _sel: *mut c_void,
    _point: NSPoint,
) -> *mut c_void {
    ptr::null_mut()
}

/// Registers (once) the NSOpenGLView subclass mpv renders into.
unsafe fn video_view_class() -> *mut c_void {
    let existing = get_class(b"VeloVideoView\0");
    if !existing.is_null() {
        return existing;
    }

    let superclass = get_class(b"NSOpenGLView\0");
    let cls =
        objc_allocateClassPair(superclass, b"VeloVideoView\0".as_ptr() as *const c_char, 0);
    if cls.is_null() {
        return superclass;
    }

    class_addMethod(
        cls,
        get_sel(b"drawRect:\0"),
        video_view_draw_rect as *const c_void,
        b"v@:{CGRect={CGPoint=dd}{CGSize=dd}}\0".as_ptr() as *const c_char,
    );
    class_addMethod(
        cls,
        get_sel(b"veloRequestRedraw\0"),
        video_view_request_redraw as *const c_void,
        b"v@:\0".as_ptr() as *const c_char,
    );
    class_addMethod(
        cls,
        get_sel(b"hitTest:\0"),
        video_view_hit_test as *const c_void,
        b"@@:{CGPoint=dd}\0".as_ptr() as *const c_char,
    );

    objc_registerClassPair(cls);
    cls
}

unsafe fn make_pixel_format() -> *mut c_void {
    let attributes: [u32; 6] = [
        NS_OPENGL_PFA_OPENGL_PROFILE,
        NS_OPENGL_PROFILE_VERSION_3_2_CORE,
        NS_OPENGL_PFA_ACCELERATED,
        NS_OPENGL_PFA_DOUBLE_BUFFER,
        NS_OPENGL_PFA_ALLOW_OFFLINE,
        0,
    ];

    let allocated = msg_obj(get_class(b"NSOpenGLPixelFormat\0"), get_sel(b"alloc\0"));
    if allocated.is_null() {
        return ptr::null_mut();
    }

    let init: unsafe extern "C" fn(*mut c_void, *mut c_void, *const u32) -> *mut c_void =
        std::mem::transmute(objc_msgSend as *const ());
    init(
        allocated,
        get_sel(b"initWithAttributes:\0"),
        attributes.as_ptr(),
    )
}

pub struct MacosPlatform {
    sleep_assertion_id: u32,
}

impl MacosPlatform {
    pub fn new() -> Self {
        Self {
            sleep_assertion_id: 0,
        }
    }

    /// Creates the OpenGL view mpv renders into and inserts it underneath the
    /// webview, inside the app's own window. Leaves the view's GL context
    /// current, which is what `mpv_render_context_create` needs.
    pub unsafe fn create_video_surface(ns_window_ptr: *mut c_void) -> Result<()> {
        if ns_window_ptr.is_null() {
            return Err(VeloError::Platform("NSWindow pointer is null".into()));
        }

        let content_view = msg_obj(ns_window_ptr, get_sel(b"contentView\0"));
        if content_view.is_null() {
            return Err(VeloError::Platform("NSWindow contentView is null".into()));
        }

        let pixel_format = make_pixel_format();
        if pixel_format.is_null() {
            return Err(VeloError::Platform("No usable OpenGL pixel format".into()));
        }

        let bounds = rect_of(content_view, b"bounds\0");
        let allocated = msg_obj(video_view_class(), get_sel(b"alloc\0"));
        if allocated.is_null() {
            return Err(VeloError::Platform("Failed to allocate video view".into()));
        }

        let init: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            NSRect,
            *mut c_void,
        ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());
        let view = init(
            allocated,
            get_sel(b"initWithFrame:pixelFormat:\0"),
            bounds,
            pixel_format,
        );
        if view.is_null() {
            return Err(VeloError::Platform("Failed to create video view".into()));
        }

        let msg_u64: unsafe extern "C" fn(*mut c_void, *mut c_void, u64) =
            std::mem::transmute(objc_msgSend as *const ());
        msg_u64(
            view,
            get_sel(b"setAutoresizingMask:\0"),
            NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE,
        );
        // Render at native pixel density rather than upscaled points.
        msg_set_bool(
            view,
            get_sel(b"setWantsBestResolutionOpenGLSurface:\0"),
            true,
        );

        // Below the webview, so the transparent HTML UI composites on top and
        // keeps receiving input.
        let add_subview: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut c_void,
            i64,
            *mut c_void,
        ) = std::mem::transmute(objc_msgSend as *const ());
        add_subview(
            content_view,
            get_sel(b"addSubview:positioned:relativeTo:\0"),
            view,
            NS_WINDOW_BELOW,
            ptr::null_mut(),
        );

        // mpv_render_context_create needs this context current on this thread.
        let gl_context = msg_obj(view, get_sel(b"openGLContext\0"));
        if gl_context.is_null() {
            return Err(VeloError::Platform("View has no OpenGL context".into()));
        }
        msg_void(gl_context, get_sel(b"makeCurrentContext\0"));

        VIDEO_VIEW.store(view, Ordering::Release);
        tracing::info!(
            "video surface created below the webview: {}x{} pt",
            bounds.size.width,
            bounds.size.height
        );
        Ok(())
    }

    /// Keeps the video surface filling the window. The autoresizing mask
    /// normally handles this; re-applying the frame covers what it misses
    /// (fullscreen transitions, display scale changes).
    pub unsafe fn sync_video_surface(ns_window_ptr: *mut c_void) {
        let view = VIDEO_VIEW.load(Ordering::Acquire);
        if view.is_null() || ns_window_ptr.is_null() {
            return;
        }

        let content_view = msg_obj(ns_window_ptr, get_sel(b"contentView\0"));
        if content_view.is_null() {
            return;
        }

        let bounds = rect_of(content_view, b"bounds\0");
        let set_frame: unsafe extern "C" fn(*mut c_void, *mut c_void, NSRect) =
            std::mem::transmute(objc_msgSend as *const ());
        set_frame(view, get_sel(b"setFrame:\0"), bounds);
        msg_set_bool(view, get_sel(b"setNeedsDisplay:\0"), true);
    }

    /// Asks the view to redraw. Safe to call from any thread -- mpv's render
    /// update callback arrives on one of its own.
    pub unsafe fn request_redraw() {
        let view = VIDEO_VIEW.load(Ordering::Acquire);
        if view.is_null() {
            return;
        }

        let perform: unsafe extern "C" fn(
            *mut c_void,
            *mut c_void,
            *mut c_void,
            *mut c_void,
            i8,
        ) = std::mem::transmute(objc_msgSend as *const ());
        perform(
            view,
            get_sel(b"performSelectorOnMainThread:withObject:waitUntilDone:\0"),
            get_sel(b"veloRequestRedraw\0"),
            ptr::null_mut(),
            0,
        );
    }

    /// Prevent macOS display from sleeping during playback.
    pub fn prevent_sleep(&mut self, prevent: bool) {
        unsafe {
            if prevent {
                if self.sleep_assertion_id == 0 {
                    let mut assertion_id: u32 = 0;
                    let class_nsstring = get_class(b"NSString\0");
                    let sel_str_with_utf8 = get_sel(b"stringWithUTF8String:\0");
                    let msg_send_str: unsafe extern "C" fn(
                        *mut c_void,
                        *mut c_void,
                        *const c_char,
                    ) -> *mut c_void = std::mem::transmute(objc_msgSend as *const ());

                    let type_str = msg_send_str(
                        class_nsstring,
                        sel_str_with_utf8,
                        b"NoDisplaySleepAssertion\0".as_ptr() as *const c_char,
                    );
                    let name_str = msg_send_str(
                        class_nsstring,
                        sel_str_with_utf8,
                        b"Velo Video Playback\0".as_ptr() as *const c_char,
                    );

                    let res = IOPMAssertionCreateWithName(
                        type_str,
                        255, // kIOPMAssertionLevelOn
                        name_str,
                        &mut assertion_id,
                    );
                    if res == 0 {
                        self.sleep_assertion_id = assertion_id;
                    }
                }
            } else if self.sleep_assertion_id != 0 {
                let _ = IOPMAssertionRelease(self.sleep_assertion_id);
                self.sleep_assertion_id = 0;
            }
        }
    }
}

extern "C" {
    fn objc_msgSend();
    #[cfg(target_arch = "x86_64")]
    fn objc_msgSend_stret();
}
