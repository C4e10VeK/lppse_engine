use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::graphics::device::PhysicalDevice;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::instance::Instance;
use crate::utils;

pub struct Surface {
    handle: ash::vk::SurfaceKHR,
    surface_fn: ash::khr::surface::Instance,
    instance: Rc<Instance>,
}

impl Debug for Surface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("raw", &self.handle)
            .field("surface_fn", &std::ptr::addr_of!(self.surface_fn))
            .field("instance", &self.instance)
            .finish()
    }
}

impl Surface {
    pub fn from_window<T>(instance: Rc<Instance>, handle: &T) -> ash::prelude::VkResult<Self>
    where
        T: HasDisplayHandle + HasWindowHandle,
    {
        let handle =
            unsafe { utils::gfx::create_surface(&instance.entry(), &instance.handle(), handle) }?;

        let surface_fn = ash::khr::surface::Instance::new(&instance.entry(), &instance.handle());

        Ok(Self {
            handle,
            surface_fn,
            instance,
        })
    }

    pub fn handle(&self) -> ash::vk::SurfaceKHR {
        self.handle
    }

    pub fn get_physical_device_surface_support(
        &self,
        physical_device: &PhysicalDevice,
        queue_index: u32,
    ) -> ash::prelude::VkResult<bool> {
        unsafe {
            self.surface_fn.get_physical_device_surface_support(
                physical_device.handle(),
                queue_index,
                self.handle,
            )
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        println!(stringify!(Surface::drop()));
        unsafe { self.surface_fn.destroy_surface(self.handle, None) };
    }
}
