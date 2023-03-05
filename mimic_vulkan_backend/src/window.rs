use core::ffi::c_void;
use std::os::raw::c_ulong;
//////////////////////// Enums ///////////////////////
pub enum WindowSurface {
    WindowsSurface {
        hinstance: *const c_void,
        hwnd: *const c_void,
    },
    MacOsSurface {
        ns_window: *const c_void,
        ns_view: *const c_void,
    },
    X11Surface {
        xlib_window: c_ulong,
        xlib_display: *const c_void,
    },
}
//////////////////////// Structs ///////////////////////
pub struct WindowSize {
    pub monitor_resolution_width: u32,
    pub monitor_resolution_height: u32,
}