use self::{
    debug_utils::{DebugUtils, DebugUtilsBuilder, DebugUtilsCallback},
    instance::{Instance, InstanceBuilder},
    surface::Surface,
};
use super::{APP_MAJOR_VERSION, APP_MINOR_VERSION, APP_NAME, APP_PATCH_VERSION};
use crate::graphics::debug_utils::DebugUtilsMessengerCallbackData;
use crate::utils::gfx::enumerate_required_extensions;
use crate::utils::make_version;
use ash::vk;
use std::rc::Rc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod debug_utils;
mod instance;
mod surface;

#[derive(Debug)]
pub struct State {
    instance: Rc<Instance>,
    _debug_utils: Option<DebugUtils>,
    surface: Rc<Surface>,
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

        let debug_utils = cfg!(feature = "gfx_debug_msg").then_some(
            DebugUtilsBuilder::default()
                .build(instance.clone())
                .expect("Error while create DebugUtilsMessenger"),
        );

        let surface =
            Surface::from_window(instance.clone(), handle).expect("Error while create surface");

        let (physical_device, properties, queue_family_index) = instance
            .enumerate_physical_devices()
            .unwrap()
            .into_iter()
            .filter_map(|pd| {
                let properties = instance.get_physical_device_properties(pd);
                instance
                    .get_physical_device_queue_family_properties(pd)
                    .into_iter()
                    .enumerate()
                    .find(|(index, qfp)| {
                        qfp.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && surface
                                .get_physical_device_surface_support(pd, *index as u32)
                                .unwrap()
                    })
                    .map(|(index, _)| (pd, properties, index as u32))
            })
            .min_by_key(|(pd, properties, _)| match properties.device_type {
                vk::PhysicalDeviceType::DISCRETE_GPU => 0,
                vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
                vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                vk::PhysicalDeviceType::CPU => 3,
                vk::PhysicalDeviceType::OTHER => 4,
                _ => 5,
            })
            .expect("No device available");

        println!(
            "Selected device: {}",
            properties.device_name_as_c_str().unwrap().to_str().unwrap()
        );

        Self {
            instance,
            _debug_utils: debug_utils,
            surface,
        }
    }
}
