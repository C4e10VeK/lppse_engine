use self::device::Device;
use self::{
    debug_utils::{DebugUtils, DebugUtilsBuilder},
    device::{DeviceBuilder, QueueDescription},
    instance::{Instance, InstanceBuilder},
    surface::Surface,
    swapchain::{Swapchain, SwapchainDescription, SwapchainImageDescription},
};
use super::{APP_MAJOR_VERSION, APP_MINOR_VERSION, APP_NAME, APP_PATCH_VERSION};
use crate::graphics::device::Queue;
use crate::utils::gfx::enumerate_required_extensions;
use crate::utils::{make_version, IntoExtent2D};
use ash::vk;
use std::rc::Rc;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};

mod debug_utils;
mod device;
mod instance;
mod surface;
mod swapchain;
mod texture;

#[derive(Debug)]
pub struct GraphicsState {
    swapchain: Rc<Swapchain>,
    queue: Queue,
    device: Rc<Device>,
    surface: Rc<Surface>,
    _debug_utils: Option<DebugUtils>,
    instance: Rc<Instance>,
}

impl GraphicsState {
    pub fn new(window: &winit::window::Window) -> Self {
        let required_extensions: Vec<_> = {
            let mut res = enumerate_required_extensions(window).unwrap();

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
            Surface::from_window(instance.clone(), window).expect("Error while create surface"),
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
                                .unwrap_or(false)
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

        let queue_description = QueueDescription::new()
            .queue_family_index(queue_family_index)
            .priority(vec![1.0f32]);

        let device_features = vk::PhysicalDeviceFeatures::default().sampler_anisotropy(true);

        let (device, mut queues) = DeviceBuilder::new()
            .queues(vec![queue_description])
            .extensions(vec![ash::khr::swapchain::NAME
                .to_str()
                .unwrap()
                .to_string()])
            .features(device_features)
            .push_extend(
                vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true),
            )
            .build(instance.clone(), physical_device)
            .expect("Error while create device");

        let queue = queues.next().unwrap();

        let cpas = device.get_surface_capabilities(&surface);
        let present_mode = {
            let modes = device.get_surface_present_modes(&surface);

            modes
                .into_iter()
                .find(|x| *x == vk::PresentModeKHR::MAILBOX)
                .unwrap()
        };

        let image_format = {
            let formats = device.get_surface_formats(&surface);

            formats
                .into_iter()
                .find(|x| {
                    x.format == vk::Format::R8G8B8A8_SRGB
                        && x.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                })
                .unwrap()
        };

        let image_description = SwapchainImageDescription {
            format: image_format.format,
            color_space: image_format.color_space,
            extent: window.inner_size().into_extent(),
            array_layers: 1,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        };

        let swapchain = {
            let swapchain = Swapchain::new(
                device.clone(),
                &surface,
                SwapchainDescription {
                    image_description,
                    present_mode,
                    min_image_count: cpas.min_image_count + 1,
                    queue_indices: vec![queue.family_index()],
                    pre_transform: cpas.current_transform,
                    composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                    ..Default::default()
                },
            )
            .expect("Error while create swapchain");

            Rc::new(swapchain)
        };

        Self {
            instance,
            _debug_utils,
            surface,
            device,
            queue,
            swapchain,
        }
    }
}
