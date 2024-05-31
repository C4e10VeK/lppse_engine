use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use super::instance::Instance;
use crate::utils;

pub struct Surface {
    raw: ash::vk::SurfaceKHR,
    surface_fn: ash::khr::surface::Instance,
    instance: Rc<Instance>,
}

impl Debug for Surface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("raw", &self.raw)
            .field("surface_fn", &"")
            .field("instance", &self.instance)
            .finish()
    }
}

impl Surface {
    pub fn from_window<T>(instance: Rc<Instance>, handle: &T) -> ash::prelude::VkResult<Rc<Self>>
    where
        T: HasDisplayHandle + HasWindowHandle,
    {
        let raw =
            unsafe { utils::gfx::create_surface(&instance.entry(), &instance.handle(), handle) }?;

        let surface_fn = ash::khr::surface::Instance::new(&instance.entry(), &instance.handle());

        Ok(Rc::new(Self { raw, surface_fn, instance }))
    }

    pub fn get_physical_device_surface_support(
        &self,
        physical_device: ash::vk::PhysicalDevice,
        queue_index: u32,
    ) -> ash::prelude::VkResult<bool> {
        unsafe {
            self.surface_fn.get_physical_device_surface_support(
                physical_device,
                queue_index,
                self.raw,
            )
        }
    }
}

impl Drop for Surface {
    #[inline]
    fn drop(&mut self) {
        unsafe { self.surface_fn.destroy_surface(self.raw, None) };
    }
}
