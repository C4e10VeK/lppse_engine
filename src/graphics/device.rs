use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use ash::vk;

use super::instance::Instance;

pub struct Device {
    handle: ash::Device,
    swap_chain_fns: ash::khr::swapchain::Device,
}

impl Device {
    pub fn new(handle: ash::Device, swap_chain_fns: ash::khr::swapchain::Device) -> Self {
        Self {
            handle,
            swap_chain_fns,
        }
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
            self.handle.device_wait_idle().unwrap();
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

#[derive(Default)]
pub struct DeviceBuilder<'a> {
    pub extensions: Vec<String>,
    pub features: vk::PhysicalDeviceFeatures,
    pub extends: Vec<Box<dyn vk::ExtendsDeviceCreateInfo + 'a>>,
    pub queues: Vec<QueueDescription>,
}

impl<'a> DeviceBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }
    pub fn features(mut self, features: vk::PhysicalDeviceFeatures) -> Self {
        self.features = features;
        self
    }
    pub fn push_extend<T: vk::ExtendsDeviceCreateInfo + 'a>(mut self, extend: T) -> Self {
        self.extends.push(Box::new(extend));
        self
    }
    pub fn queues(mut self, queues: Vec<QueueDescription>) -> Self {
        self.queues = queues;
        self
    }

    pub fn build(
        self,
        instance: Rc<Instance>,
        physical_device: PhysicalDevice,
    ) -> ash::prelude::VkResult<Rc<Device>> {
        let extensions: Vec<_> = self
            .extensions
            .into_iter()
            .map(|s| std::ffi::CString::new(s).unwrap())
            .collect();

        let extensions_raw: Vec<_> = extensions.iter().map(|s| s.as_ptr()).collect();

        let queue_create_infos: Vec<_> = self
            .queues
            .iter()
            .map(|d| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(d.queue_family_index)
                    .queue_priorities(&d.priority)
            })
            .collect();

        let mut extends = self.extends;
        let device_create_info = {
            let mut device_create_info = vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&extensions_raw)
                .enabled_features(&self.features);

            for x in extends.iter_mut() {
                device_create_info = device_create_info.push_next(x.as_mut());
            }

            device_create_info
        };

        let device = instance.create_device(physical_device.handle(), &device_create_info)?;

        let swap_chain_fns = ash::khr::swapchain::Device::new(&instance.handle(), &device);

        Ok(Rc::new(Device::new(device, swap_chain_fns)))
    }
}

#[derive(Default)]
pub struct QueueDescription {
    pub queue_family_index: u32,
    pub priority: Vec<f32>,
}

impl QueueDescription {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue_family_index(mut self, queue_family_index: u32) -> Self {
        self.queue_family_index = queue_family_index;
        self
    }

    pub fn priority(mut self, priority: Vec<f32>) -> Self {
        self.priority = priority;
        self
    }
}
