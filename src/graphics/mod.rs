use self::device::Device;
use self::{
    debug_utils::{DebugUtils, DebugUtilsBuilder},
    instance::{Instance, InstanceBuilder},
    surface::Surface,
};
use super::{APP_MAJOR_VERSION, APP_MINOR_VERSION, APP_NAME, APP_PATCH_VERSION};
use crate::utils::gfx::enumerate_required_extensions;
use crate::utils::make_version;
use ash::vk;
use std::rc::Rc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod debug_utils;
mod device;
mod instance;
mod surface;

#[derive(Debug)]
pub struct State {
    instance: Rc<Instance>,
    _debug_utils: Option<DebugUtils>,
    surface: Rc<Surface>,
    device: Rc<Device>,
}

impl State {
    pub fn new<T>(handle: &T) -> Self
    where
        T: HasDisplayHandle + HasWindowHandle,
    {
        let required_extensions: Vec<_> = {
            let mut res = enumerate_required_extensions(handle).unwrap();

            if cfg!(feature = "gfx_debug_msg") {
                res.push(ash::ext::debug_utils::NAME.to_str().unwrap().to_string());
            }

            res
        };

        let layers = {
            let mut res = vec![];

            if cfg!(feature = "gfx_debug_msg") {
                res.push("VK_LAYER_KHRONOS_validation".to_owned());
            }

            res
        };

        let app_version = make_version(
            APP_MAJOR_VERSION.parse().unwrap(),
            APP_MINOR_VERSION.parse().unwrap(),
            APP_PATCH_VERSION.parse().unwrap(),
        );

        let instance = InstanceBuilder::new()
            .application_name(APP_NAME)
            .application_version(app_version)
            .api_version(vk::API_VERSION_1_3)
            .extensions(required_extensions)
            .layers(layers)
            .build()
            .expect("Error while create instance");

        let _debug_utils = cfg!(feature = "gfx_debug_msg").then_some(
            DebugUtilsBuilder::default()
                .build(instance.clone())
                .expect("Error while create DebugUtilsMessenger"),
        );

        let surface = Rc::new(
            Surface::from_window(instance.clone(), handle).expect("Error while create surface"),
        );

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .into_iter()
            .filter_map(|pd| {
                pd.get_queue_family_properties()
                    .into_iter()
                    .enumerate()
                    .find(|(index, qfp)| {
                        qfp.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && surface
                                .get_physical_device_surface_support(&pd, *index as u32)
                                .unwrap()
                    })
                    .map(|(index, _)| (pd, index as u32))
            })
            .min_by_key(|(pd, _)| match pd.get_properties().device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                vk::PhysicalDeviceType::CPU => 3,
                vk::PhysicalDeviceType::OTHER => 4,
                _ => 5,
            })
            .expect("No device available");

        let queue_create_infos = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0f32]);

        let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];

        let device_features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);

        let binding = [queue_create_infos];
        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&binding)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&device_features);

        let result = instance
            .create_device(physical_device.handle(), &device_create_info)
            .expect("Error while create device");

        let device = Rc::new(Device::new(result));

        Self {
            instance,
            _debug_utils,
            surface,
            device,
        }
    }
}
