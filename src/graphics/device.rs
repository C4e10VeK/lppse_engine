use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use ash::vk;

use super::instance::Instance;

pub struct Device {
    handle: ash::Device,
}

impl Device {
    pub fn new(handle: ash::Device) -> Self {
        Self { handle }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle.handle())
            .finish()
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        println!(stringify!(Device::drop()));
        unsafe {
            self.handle.device_wait_idle().unwrap_or(());
            self.handle.destroy_device(None);
        }
    }
}

#[derive(Debug)]
pub struct PhysicalDevice {
    handle: vk::PhysicalDevice,
    instance: Rc<Instance>,
}

impl PhysicalDevice {
    pub fn new(handle: vk::PhysicalDevice, instance: Rc<Instance>) -> Self {
        Self { handle, instance }
    }

    pub fn handle(&self) -> vk::PhysicalDevice {
        self.handle
    }

    pub fn get_properties(&self) -> vk::PhysicalDeviceProperties {
        self.instance.get_physical_device_properties(self.handle)
    }

    pub fn get_queue_family_properties(&self) -> Vec<vk::QueueFamilyProperties> {
        self.instance
            .get_physical_device_queue_family_properties(self.handle)
    }

    pub fn get_features(&self) -> vk::PhysicalDeviceFeatures {
        self.instance.get_physical_device_features(self.handle)
    }
}
