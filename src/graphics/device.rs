use ash::prelude::VkResult;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::graphics::surface::Surface;
use ash::vk;

use super::instance::Instance;

pub struct Device {
    handle: ash::Device,
    swap_chain_fns: ash::khr::swapchain::Device,
    physical_device: vk::PhysicalDevice,
}

impl Device {
    fn new(
        handle: ash::Device,
        swap_chain_fns: ash::khr::swapchain::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        Self {
            handle,
            swap_chain_fns,
            physical_device,
        }
    }

    pub fn get_surface_capabilities(&self, surface: &Surface) -> vk::SurfaceCapabilitiesKHR {
        surface
            .get_physical_device_surface_capabilities(self.physical_device)
            .unwrap()
    }

    pub fn get_surface_formats(&self, surface: &Surface) -> Vec<vk::SurfaceFormatKHR> {
        surface
            .get_physical_device_surface_formats(self.physical_device)
            .unwrap()
    }

    pub fn get_surface_present_modes(&self, surface: &Surface) -> Vec<vk::PresentModeKHR> {
        surface
            .get_physical_device_surface_present_modes(self.physical_device)
            .unwrap()
    }

    pub fn create_swapchain(
        &self,
        create_info: &vk::SwapchainCreateInfoKHR<'_>,
    ) -> VkResult<vk::SwapchainKHR> {
        unsafe { self.swap_chain_fns.create_swapchain(create_info, None) }
    }

    pub fn destroy_swapchain(&self, swapchain: vk::SwapchainKHR) {
        unsafe { self.swap_chain_fns.destroy_swapchain(swapchain, None) }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle.handle())
            .field("swap_chain_fns", &std::ptr::addr_of!(self.swap_chain_fns))
            .field("physical_device", &self.physical_device)
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

#[derive(Debug, Copy, Clone)]
pub struct Queue {
    handle: vk::Queue,
    family_index: u32,
    index: u32,
}

impl Queue {
    fn new(handle: vk::Queue, family_index: u32, index: u32) -> Self {
        Self {
            handle,
            family_index,
            index,
        }
    }

    pub fn handle(&self) -> vk::Queue {
        self.handle
    }

    pub fn family_index(&self) -> u32 {
        self.family_index
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
    ) -> ash::prelude::VkResult<(Rc<Device>, impl ExactSizeIterator<Item = Queue>)> {
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

        let device = {
            let device = instance.create_device(physical_device.handle, &device_create_info)?;
            let swap_chain_fns = ash::khr::swapchain::Device::new(&instance.handle(), &device);
            Rc::new(Device::new(device, swap_chain_fns, physical_device.handle))
        };

        let queue_infos: Vec<_> = {
            let temp = self
                .queues
                .iter()
                .map(|x| (x.queue_family_index, 0..x.priority.len() as u32));

            let mut queue_infos = vec![];
            for (family_index, indices) in temp {
                for x in indices {
                    queue_infos.push((family_index, x));
                }
            }

            queue_infos
        };

        let queues = {
            let device_for_iter = device.clone();
            queue_infos.into_iter().map(move |(family, index)| {
                let queue = unsafe { device_for_iter.handle.get_device_queue(family, index) };
                Queue::new(queue, family, index)
            })
        };

        Ok((device, queues))
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
