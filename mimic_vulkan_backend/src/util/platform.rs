use crate::util::result::{Result, VulkanError};
use crate::window::WindowSurface;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
use ash::extensions::khr::XlibSurface;
#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::Surface;

use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;

// required extension ------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

// surfaces --------------------------------------------------------------------------
pub struct SurfaceContainer {
    pub surface_loader: ash::extensions::khr::Surface,
    pub surface: vk::SurfaceKHR,
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window_surface: &WindowSurface,
) -> Result<vk::SurfaceKHR> {
    use std::ptr;

    let (x11_window, x11_display) = if let &WindowSurface::X11Surface {
        xlib_window,
        xlib_display,
    } = window_surface
    {
        (xlib_window, xlib_display)
    } else {
        return Err(VulkanError::WindowIncorrectPlatformSurface);
    };

    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };
    let xlib_surface_loader = XlibSurface::new(entry, instance);
    let surface = unsafe { xlib_surface_loader.create_xlib_surface(&x11_create_info, None)? };
    Ok(SurfaceContainer {
        surface_loader: ash::extensions::khr::Surface::new(entry, instance),
        surface,
    })
}

#[cfg(target_os = "macos")]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window_surface: &WindowSurface,
) -> Result<vk::SurfaceKHR> {
    use std::mem;
    use std::os::raw::c_void;
    use std::ptr;

    let (ns_window, ns_view) =
        if let &WindowSurface::MacOSSurface { ns_window, ns_view } = window_surface {
            (ns_window, ns_view)
        } else {
            return Err(VulkanError::WindowIncorrectPlatformSurface);
        };

    let wnd: cocoa_id = mem::transmute(ns_window);

    let layer = CoreAnimationLayer::new();

    layer.set_edge_antialiasing_mask(0);
    layer.set_presents_with_transaction(false);
    layer.remove_all_animations();

    let view = wnd.contentView();

    layer.set_contents_scale(view.backingScaleFactor());
    view.setLayer(mem::transmute(layer.as_ref()));
    view.setWantsLayer(YES);

    let create_info = vk::MacOSSurfaceCreateInfoMVK {
        s_type: vk::StructureType::MACOS_SURFACE_CREATE_INFO_M,
        p_next: ptr::null(),
        flags: Default::default(),
        p_view: ns_view as *const c_void,
    };

    let macos_surface_loader = MacOSSurface::new(entry, instance);
    let surface = unsafe { macos_surface_loader.create_mac_os_surface_mvk(&create_info, None)? };
    Ok(SurfaceContainer {
        surface_loader: ash::extensions::khr::Surface::new(entry, instance),
        surface,
    })
}

#[cfg(target_os = "windows")]
pub fn create_surface<E: EntryV1_0, I: InstanceV1_0>(
    entry: &E,
    instance: &I,
    window_surface: &WindowSurface,
) -> Result<SurfaceContainer> {
    use std::os::raw::c_void;
    use std::ptr;
    use winapi::shared::windef::HWND;
    use winapi::um::libloaderapi::GetModuleHandleW;

    let hwnd = if let &WindowSurface::WindowsSurface { hinstance: _, hwnd } = window_surface {
        hwnd
    } else {
        return Err(VulkanError::WindowIncorrectPlatformSurface);
    } as HWND;

    let hinstance = unsafe { GetModuleHandleW(ptr::null()) as *const c_void };
    let win32_create_info = vk::Win32SurfaceCreateInfoKHR {
        s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        hinstance,
        hwnd: hwnd as *const c_void,
    };
    let win32_surface_loader = Win32Surface::new(entry, instance);
    let surface = unsafe { win32_surface_loader.create_win32_surface(&win32_create_info, None)? };
    Ok(SurfaceContainer {
        surface_loader: ash::extensions::khr::Surface::new(entry, instance),
        surface,
    })
}
