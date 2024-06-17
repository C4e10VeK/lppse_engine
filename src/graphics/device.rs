use ash::prelude::VkResult;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use crate::gfx_debug_log;
use crate::graphics::surface::Surface;
use ash::vk;

use super::instance::Instance;

pub struct Device {
    handle: ash::Device,
    _instance: Rc<Instance>,
    swapchain_fns: ash::khr::swapchain::Device,
    _dynamic_rendering_fns: ash::khr::dynamic_rendering::Device,
    physical_device: vk::PhysicalDevice,
}

impl Device {
    fn new(
        handle: ash::Device,
        instance: Rc<Instance>,
        swapchain_fns: ash::khr::swapchain::Device,
        dynamic_rendering_fns: ash::khr::dynamic_rendering::Device,
        physical_device: vk::PhysicalDevice,
    ) -> Self {
        Self {
            handle,
            _instance: instance,
            swapchain_fns,
            _dynamic_rendering_fns: dynamic_rendering_fns,
            physical_device,
        }
    }

    pub fn default_extensions() -> Vec<String> {
        vec![
            "VK_KHR_swapchain".to_owned(),
            "VK_KHR_dynamic_rendering".to_owned(),
        ]
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

    pub fn get_swapchain_images(&self, swapchain: vk::SwapchainKHR) -> VkResult<Vec<vk::Image>> {
        unsafe { self.swapchain_fns.get_swapchain_images(swapchain) }
    }

    pub fn wait_idle(&self) -> VkResult<()> {
        unsafe { self.handle.device_wait_idle() }
    }
}

impl Debug for Device {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Device")
            .field("handle", &self.handle.handle())
            .field("swap_chain_fns", &std::ptr::addr_of!(self.swapchain_fns))
            .field("physical_device", &self.physical_device)
            .finish()
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        gfx_debug_log!(stringify!(Device::drop()));
        unsafe {
            self.wait_idle().unwrap();
            self.handle.destroy_device(None);
        }
    }
}

pub trait VulkanDevice {
    fn handle(&self) -> ash::Device;
    fn swapchain_fns(&self) -> ash::khr::swapchain::Device;
}

impl VulkanDevice for Device {
    fn handle(&self) -> ash::Device {
        self.handle.clone()
    }
    fn swapchain_fns(&self) -> ash::khr::swapchain::Device {
        self.swapchain_fns.clone()
    }
}

pub trait DeviceCreateExtend<TInfo, TTarget>: VulkanDevice {
    fn create(&self, create_info: &TInfo) -> VkResult<TTarget>;
}

pub trait DeviceDestroyExtend<T>: VulkanDevice {
    fn destroy(&self, vk_struct: T);
}

#[derive(Debug, Clone)]
pub struct Queue {
    handle: vk::Queue,
    device: Rc<Device>,
    family_index: u32,
    index: u32,
}

impl Queue {
    fn new(handle: vk::Queue, device: Rc<Device>, family_index: u32, index: u32) -> Self {
        Self {
            handle,
            device,
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

    pub fn submit(&self, submits: &[vk::SubmitInfo<'_>], fence: vk::Fence) -> VkResult<()> {
        unsafe {
            self.device
                .handle()
                .queue_submit(self.handle, submits, fence)
        }
    }

    pub fn present(&self, present_info: vk::PresentInfoKHR<'_>) -> VkResult<bool> {
        unsafe {
            self.device
                .swapchain_fns()
                .queue_present(self.handle, &present_info)
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

pub struct DeviceBuilder<'a> {
    pub extensions: Vec<String>,
    pub features: vk::PhysicalDeviceFeatures,
    pub extends: Vec<Box<dyn vk::ExtendsDeviceCreateInfo + 'a>>,
    pub queues: Vec<QueueDescription>,
}

impl<'a> Default for DeviceBuilder<'a> {
    fn default() -> Self {
        Self {
            extensions: Device::default_extensions(),
            features: vk::PhysicalDeviceFeatures::default(),
            extends: Vec::default(),
            queues: Vec::default(),
        }
    }
}

impl<'a> DeviceBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
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
    ) -> VkResult<(Rc<Device>, impl ExactSizeIterator<Item = Queue>)> {
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
            let swapchain_fns = ash::khr::swapchain::Device::new(&instance.handle(), &device);
            let dynamic_rendering_fns =
                ash::khr::dynamic_rendering::Device::new(&instance.handle(), &device);
            Rc::new(Device::new(
                device,
                instance,
                swapchain_fns,
                dynamic_rendering_fns,
                physical_device.handle,
            ))
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
                Queue::new(queue, device_for_iter.clone(), family, index)
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

pub trait DeviceExtensionPush {
    fn push_extension(self, extension_name: impl Into<String>) -> Self;
}

impl DeviceExtensionPush for DeviceBuilder<'_> {
    fn push_extension(mut self, extension_name: impl Into<String>) -> Self {
        self.extensions.push(extension_name.into());
        self
    }
}
