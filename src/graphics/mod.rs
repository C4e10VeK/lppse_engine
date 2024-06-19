use self::{
    debug_utils::{DebugUtils, DebugUtilsBuilder},
    device::{Device, DeviceBuilder, Queue, QueueDescription, VulkanDevice},
    instance::{Instance, InstanceBuilder},
    surface::Surface,
    swapchain::{Swapchain, SwapchainDescription, SwapchainImageDescription},
    sync::{
        fence::Fence, semaphore::Semaphore, submit_task, task_from_runner, GPUTask, SubmitInfo,
    },
};
use super::{APP_MAJOR_VERSION, APP_MINOR_VERSION, APP_NAME, APP_PATCH_VERSION};
use crate::utils::gfx::enumerate_required_extensions;
use crate::utils::{make_version, IntoExtent2D};
use ash::vk;
use std::rc::Rc;
use winit::dpi::PhysicalSize;

mod debug_utils;
mod device;
mod instance;
mod surface;
mod swapchain;
mod sync;
mod texture;

const MAX_FRAMES_IN_FLIGHT: u32 = 3;

#[derive(Debug)]
pub struct GraphicsState {
    swapchain: Option<Swapchain>,
    queue: Queue,
    device: Rc<Device>,
    surface: Rc<Surface>,
    _debug_utils: Option<DebugUtils>,
    _instance: Rc<Instance>,

    present_semaphores: Vec<Semaphore>,
    render_semaphores: Vec<Semaphore>,
    fences: Vec<Fence>,
    _command_pools: Vec<vk::CommandPool>,
    command_buffers: Vec<vk::CommandBuffer>,
    current_frame: u32,
}

impl GraphicsState {
    pub fn new(window: &winit::window::Window) -> Self {
        let instance = create_instance(window);

        let _debug_utils = if cfg!(feature = "gfx_debug_msg") {
            Some(
                DebugUtilsBuilder::new()
                    .build(instance.clone())
                    .expect("Error while create DebugUtilsMessenger"),
            )
        } else {
            None
        };

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
            .features(device_features)
            .push_extend(
                vk::PhysicalDeviceDynamicRenderingFeatures::default().dynamic_rendering(true),
            )
            .build(instance.clone(), physical_device)
            .expect("Error while create device");

        let queue = queues.next().unwrap();

        let (present_semaphores, render_semaphores) = {
            let present_semaphores = (0..MAX_FRAMES_IN_FLIGHT)
                .map(|_| Semaphore::new(device.clone()).unwrap())
                .collect();

            let render_semaphores = (0..MAX_FRAMES_IN_FLIGHT)
                .map(|_| Semaphore::new(device.clone()).unwrap())
                .collect();

            (present_semaphores, render_semaphores)
        };

        let fences = (0..MAX_FRAMES_IN_FLIGHT)
            .map(|_| Fence::new(device.clone(), false).unwrap())
            .collect();

        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let (command_pools, command_buffers) = unsafe {
            let mut command_pools = vec![];
            let mut command_buffers = vec![];
            for _ in 0..MAX_FRAMES_IN_FLIGHT {
                let command_pool = device
                    .handle()
                    .create_command_pool(&command_pool_info, None)
                    .expect("Error while create command pool");

                let command_buffer = {
                    let command_buffer_info = vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .command_buffer_count(1)
                        .level(vk::CommandBufferLevel::PRIMARY);

                    device
                        .handle()
                        .allocate_command_buffers(&command_buffer_info)
                        .expect("Error while allocate command buffer")
                        .get(0)
                        .unwrap()
                        .to_owned()
                };

                command_pools.push(command_pool);
                command_buffers.push(command_buffer);
            }

            (command_pools, command_buffers)
        };

        Self {
            _instance: instance,
            _debug_utils,
            surface,
            device,
            queue,
            swapchain: None,
            present_semaphores,
            render_semaphores,
            fences,
            _command_pools: command_pools,
            command_buffers,
            current_frame: 0,
        }
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        let swapchain = create_swapchain(
            self.device.clone(),
            &self.surface,
            new_size.into_extent(),
            self.swapchain.as_ref().map(|t| t.handle()),
        );

        self.swapchain = Some(swapchain);
    }

    pub fn render(&mut self) {
        let current_fence = &self.fences[self.current_frame as usize];

        let swapchain = self.swapchain.as_ref().unwrap();

        let image_result = swapchain.get_current_image(
            self.present_semaphores[self.current_frame as usize].handle(),
            Some(current_fence.handle()),
        );

        current_fence.wait(u64::MAX).unwrap();
        current_fence.reset();

        let (current_image, image_index) = match image_result {
            Ok((current_image, image_index, suboptimal)) => {
                if suboptimal {
                    return;
                }

                (current_image, image_index)
            }
            Err(_) => {
                return;
            }
        };

        let current_command_buffer = self.command_buffers[self.current_frame as usize];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(current_command_buffer, &command_buffer_begin_info)
                .unwrap();

            // Begin rendering
            let image_barrier = [vk::ImageMemoryBarrier::default()
                .image(current_image.image())
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })];

            self.device.handle().cmd_pipeline_barrier(
                current_command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barrier,
            );

            let color_attachment = vk::RenderingAttachmentInfoKHR::default()
                .image_view(current_image.image_view())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .clear_value(vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [0.1, 0.2, 1.0, 1.0],
                    },
                });

            let render_extent = vk::Extent2D {
                width: swapchain.extent().width,
                height: swapchain.extent().height,
            };
            let rendering_info = vk::RenderingInfoKHR::default()
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: render_extent,
                })
                .layer_count(1)
                .color_attachments(std::slice::from_ref(&color_attachment));

            self.device
                .handle()
                .cmd_begin_rendering(current_command_buffer, &rendering_info);

            // End rendering
            self.device
                .handle()
                .cmd_end_rendering(current_command_buffer);

            let image_barrier = [vk::ImageMemoryBarrier::default()
                .image(current_image.image())
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::empty())
                .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })];

            self.device.handle().cmd_pipeline_barrier(
                current_command_buffer,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barrier,
            );

            // End record command buffer
            self.device
                .handle()
                .end_command_buffer(current_command_buffer)
                .unwrap();
        }

        let wait_semaphore = self.present_semaphores[self.current_frame as usize].handle();
        let signal_semaphore = self.render_semaphores[self.current_frame as usize].handle();
        let wait_dst_stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

        let submit_runner = submit_task::submit(
            self.queue.clone(),
            SubmitInfo {
                signal_semaphore,
                wait_semaphore,
                wait_dst_stage_mask,
                command_buffers: vec![current_command_buffer],
            },
            Some(current_fence.as_shared()),
        );

        let submit_task = task_from_runner(submit_runner).unwrap();

        submit_task.wait().unwrap();

        let raw_sc = swapchain.handle();

        let present_result = submit_task.then_present(self.queue.clone(), raw_sc, image_index);

        match present_result {
            Ok(suboptimal) => {
                if suboptimal {
                    return;
                }
            }
            Err(_) => {
                return;
            }
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

fn create_instance(window: &winit::window::Window) -> Rc<Instance> {
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

    InstanceBuilder::new()
        .application_name(APP_NAME)
        .application_version(app_version)
        .api_version(vk::API_VERSION_1_3)
        .extensions(required_extensions)
        .layers(layers)
        .build()
        .expect("Error while create instance")
}

#[allow(clippy::too_many_arguments)]
fn create_swapchain(
    device: Rc<Device>,
    surface: &Surface,
    extent: vk::Extent2D,
    old_swapchain: Option<vk::SwapchainKHR>,
) -> Swapchain {
    device.wait_idle().unwrap();

    let capabilities = device.get_surface_capabilities(&surface);
    let present_mode = {
        let modes = device.get_surface_present_modes(&surface);

        modes
            .into_iter()
            .find(|x| *x == vk::PresentModeKHR::MAILBOX || *x == vk::PresentModeKHR::IMMEDIATE)
            .unwrap()
    };

    let image_format = device
        .get_surface_formats(&surface)
        .into_iter()
        .find(|x| {
            x.format == vk::Format::B8G8R8A8_SRGB
                && x.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap();

    let image_description = SwapchainImageDescription {
        format: image_format.format,
        color_space: image_format.color_space,
        extent,
        array_layers: 1,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
    };

    let swapchain = {
        Swapchain::new(
            device.clone(),
            &surface,
            SwapchainDescription {
                image_description,
                present_mode,
                min_image_count: capabilities.min_image_count + 1,
                pre_transform: capabilities.current_transform,
                composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
                old_swapchain,
                ..Default::default()
            },
        )
        .expect("Error while create swapchain")
    };

    swapchain
}
