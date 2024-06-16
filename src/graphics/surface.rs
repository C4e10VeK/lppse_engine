use super::device::PhysicalDevice;
use super::instance::Instance;
use crate::graphics::swapchain::Swapchain;
use crate::{debug_log, utils};
use ash::prelude::VkResult;
use ash::vk;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::RwLock;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct Surface {
    // swapchain: RwLock<Option<Swapchain>>,
    handle: vk::SurfaceKHR,
    surface_fn: ash::khr::surface::Instance,
    instance: Rc<Instance>,
}

impl Surface {
    pub fn from_window<T>(instance: Rc<Instance>, window_handle: &T) -> VkResult<Self>
    where
        T: HasDisplayHandle + HasWindowHandle,
    {
        let handle = unsafe {
            utils::gfx::create_surface(&instance.entry(), &instance.handle(), window_handle)
        }?;

        let surface_fn = ash::khr::surface::Instance::new(&instance.entry(), &instance.handle());

        Ok(Self {
            // swapchain: RwLock::new(None),
            handle,
            surface_fn,
            instance,
        })
    }

    pub fn handle(&self) -> vk::SurfaceKHR {
        self.handle
    }

    pub fn get_physical_device_surface_support(
        &self,
        physical_device: &PhysicalDevice,
        queue_index: u32,
    ) -> VkResult<bool> {
        unsafe {
            self.surface_fn.get_physical_device_surface_support(
                physical_device.handle(),
                queue_index,
                self.handle,
            )
        }
    }

    pub fn get_physical_device_surface_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.surface_fn
                .get_physical_device_surface_capabilities(physical_device, self.handle)
        }
    }

    pub fn get_physical_device_surface_formats(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.surface_fn
                .get_physical_device_surface_formats(physical_device, self.handle)
        }
    }

    pub fn get_physical_device_surface_present_modes(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            self.surface_fn
                .get_physical_device_surface_present_modes(physical_device, self.handle)
        }
    }

    // pub fn configure(&self, device: &Rc<super::device::Device>, extent: vk::Extent2D) {
    //     let mut sw = self.swapchain.write().unwrap();
    //
    //     let old_swapchain = sw.as_ref().map(|x| x.handle());
    //
    //     *sw = Some(super::create_swapchain(device.clone(), self, extent, old_swapchain))
    // }
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

impl Drop for Surface {
    fn drop(&mut self) {
        debug_log!(stringify!(Surface::drop()));
        unsafe { self.surface_fn.destroy_surface(self.handle, None) };
    }
}
