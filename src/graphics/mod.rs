use self::{
    debug_utils::{DebugUtils, DebugUtilsBuilder, DebugUtilsCallback},
    instance::{Instance, InstanceBuilder},
    surface::Surface,
};
use super::{APP_NAME, APP_MAJOR_VERSION, APP_MINOR_VERSION, APP_PATCH_VERSION};
use crate::utils::gfx::enumerate_required_extensions;
use ash::vk;
use std::rc::Rc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use crate::utils::make_version;

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
            DebugUtilsBuilder::new()
                .debug_utils_callback(unsafe {
                    DebugUtilsCallback::new(move |severity, message_type, data| {
                        println!("{:?}::{:?}: {}", severity, message_type, data.message)
                    })
                })
                .build(instance.clone())
                .expect("Error while create DebugUtilsMessenger"),
        );

        let surface = Surface::from_window(instance.clone(), handle)
            .expect("Error while create surface");

        Self {
            instance,
            _debug_utils: debug_utils,
            surface,
        }
    }
}
