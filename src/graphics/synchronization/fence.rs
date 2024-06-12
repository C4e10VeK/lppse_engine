use crate::graphics::device::{Device, DeviceCreateExtend, DeviceDestroyExtend};
use ash::prelude::VkResult;
use ash::vk;
use std::rc::Rc;

#[derive(Debug)]
pub struct Fence {
    handle: vk::Fence,
    device: Rc<Device>,
}

impl Fence {
    pub fn new(device: Rc<Device>, signaled: bool) -> VkResult<Self> {
        let mut create_info = vk::FenceCreateInfo::default();

        if signaled {
            create_info.flags = vk::FenceCreateFlags::SIGNALED;
        }

        let handle = device.create(&create_info)?;

        Ok(Self {
            handle,
            device,
        })
    }
    
    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn reset(&self) {
        self.device.reset(self.handle).expect("Error while reset fence");
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.device.destroy(self.handle);
    }
}

pub trait DeviceFenceFns {
    fn reset(&self, fence: vk::Fence) -> VkResult<()>;
    fn reset_fences(&self, fences: &[Fence]) -> VkResult<()>;
}

impl DeviceCreateExtend<vk::FenceCreateInfo<'_>, vk::Fence> for Device {
    fn create(&self, create_info: &vk::FenceCreateInfo<'_>) -> VkResult<vk::Fence> {
        unsafe { self.handle().create_fence(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::Fence> for Device {
    fn destroy(&self, vk_struct: vk::Fence) {
        unsafe { self.handle().destroy_fence(vk_struct, None) }
    }
}

impl DeviceFenceFns for Device {
    fn reset(&self, fence: vk::Fence) -> VkResult<()> {
        unsafe { self.handle().reset_fences(std::slice::from_ref(&fence)) }
    }

    fn reset_fences(&self, fences: &[Fence]) -> VkResult<()> {
        let raw_fences: Vec<_> = fences.iter().map(|x| x.handle).collect();

        unsafe { self.handle().reset_fences(&raw_fences) }
    }
}
