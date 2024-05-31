use self::instance::{Instance, InstanceBuilder};
use self::surface::Surface;
use super::{APP_NAME, APP_VERSION};
use crate::utils::gfx::enumerate_required_extensions;
use ash::vk;
use std::rc::Rc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod instance;
mod debug_utils;
mod surface;

#[derive(Debug)]
pub struct State {
    instance: Rc<Instance>,
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
                res.push(ash::ext::debug_utils::NAME.to_str().unwrap().to_owned());
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

        let instance = InstanceBuilder::new()
            .application_name(APP_NAME)
            .application_version(APP_VERSION)
            .api_version(vk::API_VERSION_1_3)
            .extensions(required_extensions)
            .layers(layers)
            .build()
            .expect("Error while create instance");

        let surface = Surface::from_window(instance.clone(), handle)
            .expect("Error while create surface");

        // TODO: бялть я ебал в рот этот дебаг в пукане. Сделать эту хуйню

        Self { instance, surface }
    }
}
