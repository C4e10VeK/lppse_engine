use super::device::PhysicalDevice;
use ash::vk;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

pub struct Instance {
    entry: ash::Entry,
    handle: ash::Instance,
}

impl Instance {
    fn new(entry: ash::Entry, instance: ash::Instance) -> Self {
        Self {
            entry,
            handle: instance,
        }
    }

    pub fn handle(&self) -> ash::Instance {
        self.handle.clone()
    }

    pub fn entry(&self) -> ash::Entry {
        self.entry.clone()
    }

    pub fn enumerate_physical_devices(
        self: &Rc<Self>,
    ) -> ash::prelude::VkResult<Vec<PhysicalDevice>> {
        let result = unsafe { self.handle.enumerate_physical_devices() };

        result.map(|physical_devices| {
            physical_devices
                .into_iter()
                .map(|pd| PhysicalDevice::new(pd, self.clone()))
                .collect()
        })
    }

    pub fn get_physical_device_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceProperties {
        unsafe { self.handle.get_physical_device_properties(physical_device) }
    }

    pub fn get_physical_device_queue_family_properties(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Vec<vk::QueueFamilyProperties> {
        unsafe {
            self.handle
                .get_physical_device_queue_family_properties(physical_device)
        }
    }

    pub fn get_physical_device_features(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> vk::PhysicalDeviceFeatures {
        unsafe { self.handle.get_physical_device_features(physical_device) }
    }

    pub fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo<'_>,
    ) -> ash::prelude::VkResult<ash::Device> {
        unsafe {
            self.handle
                .create_device(physical_device, create_info, None)
        }
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        println!(stringify!(Instance::drop()));
        unsafe {
            self.handle.destroy_instance(None);
        }
    }
}

impl Debug for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Instance")
            .field("entry", &std::ptr::addr_of!(self.entry))
            .field("instance", &self.handle.handle())
            .finish()
    }
}

#[derive(Debug)]
pub enum InstanceBuildError {
    EntryLoad(ash::LoadingError),
    InstanceCreate(vk::Result),
}

impl Display for InstanceBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            InstanceBuildError::EntryLoad(err) => write!(f, "{err}"),
            InstanceBuildError::InstanceCreate(err) => write!(f, "{err}"),
        }
    }
}

impl Error for InstanceBuildError {}

#[derive(Debug, Default)]
pub struct InstanceBuilder {
    pub application_name: String,
    pub application_version: u32,
    pub api_version: u32,
    pub extensions: Vec<String>,
    pub layers: Vec<String>,
}

impl InstanceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn application_name<T: Into<String>>(mut self, name: T) -> Self {
        self.application_name = name.into();
        self
    }

    pub fn application_version(mut self, version: u32) -> Self {
        self.application_version = version;
        self
    }

    pub fn api_version(mut self, version: u32) -> Self {
        self.api_version = version;
        self
    }

    pub fn extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    pub fn layers(mut self, layers: Vec<String>) -> Self {
        self.layers = layers;
        self
    }

    pub fn build(self) -> Result<Rc<Instance>, InstanceBuildError> {
        let entry = unsafe { ash::Entry::load().map_err(|e| InstanceBuildError::EntryLoad(e))? };

        let app_name = std::ffi::CString::new(self.application_name).unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .engine_name(&app_name)
            .application_version(self.application_version)
            .engine_version(self.application_version)
            .api_version(self.api_version);

        let extensions: Vec<_> = self
            .extensions
            .into_iter()
            .map(|s| std::ffi::CString::new(s).unwrap())
            .collect();

        let extensions_raw: Vec<_> = extensions.iter().map(|s| s.as_ptr()).collect();

        let layers: Vec<_> = self
            .layers
            .into_iter()
            .map(|s| std::ffi::CString::new(s).unwrap())
            .collect();

        let layers_raw: Vec<_> = layers.iter().map(|s| s.as_ptr()).collect();

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions_raw)
            .enabled_layer_names(&layers_raw);

        let instance = unsafe {
            entry
                .create_instance(&instance_info, None)
                .map_err(|e| InstanceBuildError::InstanceCreate(e))?
        };

        Ok(Rc::new(Instance::new(entry, instance)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_creation() -> Result<(), InstanceBuildError> {
        let _ = InstanceBuilder::new()
            .application_name("hello world")
            .application_version(10)
            .api_version(vk::API_VERSION_1_3)
            .extensions(vec![])
            .layers(vec![])
            .build()?;

        Ok(())
    }

    #[test]
    fn test_debug_format() {
        let instance = InstanceBuilder::new()
            .application_name("hello world")
            .application_version(10)
            .api_version(vk::API_VERSION_1_3)
            .extensions(vec![])
            .layers(vec![])
            .build()
            .unwrap();

        let instance_string = format!("{:?}", instance);

        println!("{instance_string}");
    }
}
