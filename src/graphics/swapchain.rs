use super::device::Device;
use super::surface::Surface;
use ash::vk;
use std::rc::Rc;

#[derive(Debug)]
pub struct Swapchain {
    handle: vk::SwapchainKHR,
    device: Rc<Device>,

    image_format: vk::Format,
    present_mode: vk::PresentModeKHR,
    extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(
        device: Rc<Device>,
        surface: &Surface,
        description: SwapchainDescription,
    ) -> ash::prelude::VkResult<Self> {
        let create_info = {
            let mut create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(surface.handle())
                .min_image_count(description.min_image_count)
                .image_format(description.image_description.format)
                .image_color_space(description.image_description.color_space)
                .image_extent(description.image_description.extent)
                .image_array_layers(description.image_description.array_layers)
                .image_usage(description.image_description.image_usage)
                .image_sharing_mode(description.image_description.sharing_mode)
                .queue_family_indices(&description.queue_indices)
                .pre_transform(description.pre_transform)
                .composite_alpha(description.composite_alpha)
                .present_mode(description.present_mode);

            if let Some(old_swapchain) = description.old_swapchain {
                create_info.old_swapchain = old_swapchain;
            }

            create_info
        };

        let swapchain = device.create_swapchain(&create_info)?;

        Ok(Self {
            handle: swapchain,
            device,
            image_format: description.image_description.format,
            present_mode: description.present_mode,
            extent: description.image_description.extent,
        })
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.device.destroy_swapchain(self.handle);
    }
}

#[derive(Debug, Default)]
pub struct SwapchainImageDescription {
    pub format: vk::Format,
    pub color_space: vk::ColorSpaceKHR,
    pub extent: vk::Extent2D,
    pub array_layers: u32,
    pub sharing_mode: vk::SharingMode,
    pub image_usage: vk::ImageUsageFlags,
}

#[derive(Debug, Default)]
pub struct SwapchainDescription {
    pub min_image_count: u32,
    pub image_description: SwapchainImageDescription,
    pub queue_indices: Vec<u32>,
    pub pre_transform: vk::SurfaceTransformFlagsKHR,
    pub composite_alpha: vk::CompositeAlphaFlagsKHR,
    pub present_mode: vk::PresentModeKHR,
    pub old_swapchain: Option<vk::SwapchainKHR>,
}
