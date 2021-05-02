use mimic_vulkan_backend::{
    util::result::{Result, VulkanError},
    window::{WindowSize, WindowSurface},
};

pub fn get_window_size_from_winit(winit_window: &winit::window::Window) -> Result<WindowSize> {
    if let Some(current_monitor) = winit_window.current_monitor() {
        Ok(WindowSize {
            monitor_resolution_width: current_monitor.size().width,
            monitor_resolution_height: current_monitor.size().height,
        })
    } else {
        Err(VulkanError::WindowCreateFailure)
    }
}

pub fn get_window_surface_from_winit(winit_window: &winit::window::Window) -> Result<WindowSurface> {

    #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
    let surface = {
        use winit::platform::unix::WindowExtUnix;
        let xlib_window = winit_window.xlib_window().ok_or(VulkanError::WindowCreateFailure);
        let xlib_display = winit_window.xlib_display().ok_or(VulkanError::WindowCreateFailure);
        WindowSurface::X11Surface { 
            xlib_window,
            xlib_display,
        }
    };
    #[cfg(target_os = "macos")]
    let surface = {
        use winit::platform::macos::WindowExtMacOS;
        let ns_window = winit_window.ns_window();
        let ns_view = winit_window.ns_view();
        WindowSurface::MacOsSurface {
            ns_window,
            ns_view,
        }
    };
    #[cfg(target_os = "windows")]
    let surface = {            
        use winit::platform::windows::WindowExtWindows;
        let hinstance = winit_window.hinstance();
        let hwnd = winit_window.hwnd();
        WindowSurface::WindowsSurface {
            hinstance,
            hwnd,
        }
    };

    Ok(surface)
}