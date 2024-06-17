use crate::debug_log;
use crate::graphics::device::{Device, DeviceCreateExtend, DeviceDestroyExtend, VulkanDevice};
use ash::prelude::VkResult;
use ash::vk;
use std::rc::Rc;

#[derive(Debug)]
pub struct Semaphore {
    handle: vk::Semaphore,
    device: Rc<Device>,
}

impl Semaphore {
    pub fn new(device: Rc<Device>) -> VkResult<Self> {
        let create_info = vk::SemaphoreCreateInfo::default();
        let handle = device.create(&create_info)?;

        Ok(Self { handle, device })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        debug_log!(stringify!(Semaphore::drop()));
        self.device.destroy(self.handle);
    }
}

impl DeviceCreateExtend<vk::SemaphoreCreateInfo<'_>, vk::Semaphore> for Device {
    fn create(&self, create_info: &vk::SemaphoreCreateInfo<'_>) -> VkResult<vk::Semaphore> {
        unsafe { self.handle().create_semaphore(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::Semaphore> for Device {
    fn destroy(&self, vk_struct: vk::Semaphore) {
        self.wait_idle().unwrap();
        unsafe {
            self.handle().destroy_semaphore(vk_struct, None);
        }
    }
}
