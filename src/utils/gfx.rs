use ash::khr::{wayland_surface, win32_surface, xcb_surface, xlib_surface};
use ash::prelude::VkResult;
use ash::vk::{
    self, SurfaceKHR, WaylandSurfaceCreateInfoKHR, Win32SurfaceCreateInfoKHR,
    XcbSurfaceCreateInfoKHR, XlibSurfaceCreateInfoKHR,
};
use ash::{Entry, Instance};

use winit::raw_window_handle::{
    HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
};

pub unsafe fn create_surface<T>(
    entry: &Entry,
    instance: &Instance,
    handle: &T,
) -> VkResult<SurfaceKHR>
where
    T: HasDisplayHandle + HasWindowHandle,
{
    let raw_display_handle = handle.display_handle().unwrap().as_raw();
    let raw_window_handle = handle.window_handle().unwrap().as_raw();

    match (raw_display_handle, raw_window_handle) {
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
            let surface_info = XlibSurfaceCreateInfoKHR::default()
                .dpy(
                    display
                        .display
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .as_ptr(),
                )
                .window(window.window);

            let surface_fns = xlib_surface::Instance::new(entry, instance);

            surface_fns.create_xlib_surface(&surface_info, None)
        }
        (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
            let surface_info = XcbSurfaceCreateInfoKHR::default()
                .connection(
                    display
                        .connection
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .as_ptr(),
                )
                .window(window.window.get());

            let surface_fns = xcb_surface::Instance::new(entry, instance);

            surface_fns.create_xcb_surface(&surface_info, None)
        }
        (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
            let surface_info = WaylandSurfaceCreateInfoKHR::default()
                .display(display.display.as_ptr())
                .surface(window.surface.as_ptr());

            let surface_fns = wayland_surface::Instance::new(entry, instance);

            surface_fns.create_wayland_surface(&surface_info, None)
        }
        (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
            let surface_info = Win32SurfaceCreateInfoKHR::default()
                .hwnd(window.hwnd.get())
                .hinstance(
                    window
                        .hinstance
                        .ok_or(vk::Result::ERROR_INITIALIZATION_FAILED)?
                        .get(),
                );

            let surface_fns = win32_surface::Instance::new(entry, instance);

            surface_fns.create_win32_surface(&surface_info, None)
        }
        (_, _) => Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    }
}

pub fn enumerate_required_extensions<T>(handle: &T) -> VkResult<Vec<String>>
where
    T: HasDisplayHandle,
{
    let raw_display_handle = handle.display_handle().unwrap().as_raw();

    let surface_extension = ash::khr::surface::NAME.to_str().unwrap().to_owned();
    match raw_display_handle {
        RawDisplayHandle::Xlib(_) => Ok(vec![
            surface_extension,
            xlib_surface::NAME.to_str().unwrap().to_owned(),
        ]),
        RawDisplayHandle::Xcb(_) => Ok(vec![
            surface_extension,
            xcb_surface::NAME.to_str().unwrap().to_owned(),
        ]),
        RawDisplayHandle::Wayland(_) => Ok(vec![
            surface_extension,
            wayland_surface::NAME.to_str().unwrap().to_owned(),
        ]),
        RawDisplayHandle::Windows(_) => Ok(vec![
            surface_extension,
            win32_surface::NAME.to_str().unwrap().to_owned(),
        ]),
        _ => Err(vk::Result::ERROR_EXTENSION_NOT_PRESENT),
    }
}
