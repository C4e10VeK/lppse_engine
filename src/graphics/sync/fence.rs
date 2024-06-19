use crate::graphics::device::{Device, DeviceCreateExtend, DeviceDestroyExtend, VulkanDevice};
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

        Ok(Self { handle, device })
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn wait(&self, timeout: u64) -> VkResult<()> {
        unsafe {
            self.device
                .handle()
                .wait_for_fences(std::slice::from_ref(&self.handle), true, timeout)
        }
    }

    pub fn reset(&self) {
        self.device
            .reset(self.handle)
            .expect("Error while reset fence");
    }

    pub fn as_shared(&self) -> SharedFence {
        SharedFence::from(self)
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        self.device.destroy(self.handle);
    }
}

#[derive(Debug, Clone)]
pub struct SharedFence {
    handle: vk::Fence,
    device: Rc<Device>,
}

impl SharedFence {
    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn from(fence: &Fence) -> Self {
        Self {
            handle: fence.handle,
            device: fence.device.clone(),
        }
    }

    pub fn wait(&self, timeout: u64) -> VkResult<()> {
        unsafe {
            self.device
                .handle()
                .wait_for_fences(std::slice::from_ref(&self.handle), true, timeout)
        }
    }

    pub fn reset(&self) {
        self.device
            .reset(self.handle)
            .expect("Error while reset fence");
    }
}

pub trait DeviceFenceFns {
    fn reset(&self, fence: vk::Fence) -> VkResult<()>;
    fn reset_fences(&self, fences: &[Fence]) -> VkResult<()>;
    fn wait_all(&self, fences: &[Fence], wait_all: bool, timeout: u64) -> VkResult<()>;
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
        let raw_fences = get_raw_fences(fences);

        unsafe { self.handle().reset_fences(&raw_fences) }
    }

    fn wait_all(&self, fences: &[Fence], wait_all: bool, timeout: u64) -> VkResult<()> {
        let raw_fences = get_raw_fences(fences);

        unsafe {
            self.handle()
                .wait_for_fences(&raw_fences, wait_all, timeout)
        }
    }
}

#[inline]
fn get_raw_fences(fences: &[Fence]) -> Vec<vk::Fence> {
    fences.iter().map(|fence| fence.handle).collect()
}
