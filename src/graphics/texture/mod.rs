use super::device::{Device, DeviceCreateExtend, DeviceDestroyExtend, VulkanDevice};
use ash::prelude::VkResult;
use ash::vk;
use std::rc::Rc;

pub mod swapchain_image;

#[derive(Debug, Clone)]
pub struct Image {
    extent: vk::Extent3D,
    image: vk::Image,
    image_view: vk::ImageView,
    sampler: Option<vk::Sampler>,
    layer_count: u32,
    device: Rc<Device>,
}

impl Image {
    pub fn image(&self) -> vk::Image {
        self.image
    }

    pub fn image_view(&self) -> vk::ImageView {
        self.image_view
    }

    pub fn sampler(&self) -> Option<vk::Sampler> {
        self.sampler
    }

    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }
}

#[derive(Debug)]
pub struct ImageDescription {
    pub extent: vk::Extent3D,
    pub format: vk::Format,
    pub view_type: vk::ImageViewType,
    pub aspect_flags: vk::ImageAspectFlags,
    pub tiling: vk::ImageTiling,
    pub usage: vk::ImageUsageFlags,
    pub properties: vk::MemoryPropertyFlags,
    pub level_count: u32,
    pub layer_count: u32,
}

impl ImageDescription {
    pub fn image2d() -> Self {
        Self {
            view_type: vk::ImageViewType::TYPE_2D,
            aspect_flags: vk::ImageAspectFlags::COLOR,
            tiling: vk::ImageTiling::OPTIMAL,
            usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            properties: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            ..Self::default()
        }
    }

    pub fn image_depth() -> Self {
        Self {
            aspect_flags: vk::ImageAspectFlags::DEPTH,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            ..Self::image2d()
        }
    }

    pub fn extent(mut self, extent: vk::Extent3D) -> Self {
        self.extent = extent;
        self
    }

    pub fn format(mut self, format: vk::Format) -> Self {
        self.format = format;
        self
    }
}

impl Default for ImageDescription {
    fn default() -> Self {
        Self {
            extent: vk::Extent3D::default(),
            format: vk::Format::default(),
            view_type: vk::ImageViewType::default(),
            aspect_flags: vk::ImageAspectFlags::default(),
            tiling: vk::ImageTiling::default(),
            usage: vk::ImageUsageFlags::default(),
            properties: vk::MemoryPropertyFlags::default(),
            level_count: 1,
            layer_count: 1,
        }
    }
}

impl DeviceCreateExtend<vk::ImageCreateInfo<'_>, vk::Image> for Device {
    fn create(&self, create_info: &vk::ImageCreateInfo<'_>) -> VkResult<vk::Image> {
        unsafe { self.handle().create_image(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::Image> for Device {
    fn destroy(&self, vk_struct: vk::Image) {
        unsafe {
            self.handle().destroy_image(vk_struct, None);
        }
    }
}

impl DeviceCreateExtend<vk::ImageViewCreateInfo<'_>, vk::ImageView> for Device {
    fn create(&self, create_info: &vk::ImageViewCreateInfo<'_>) -> VkResult<vk::ImageView> {
        unsafe { self.handle().create_image_view(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::ImageView> for Device {
    fn destroy(&self, vk_struct: vk::ImageView) {
        unsafe {
            self.handle().destroy_image_view(vk_struct, None);
        }
    }
}

impl DeviceCreateExtend<vk::SamplerCreateInfo<'_>, vk::Sampler> for Device {
    fn create(&self, create_info: &vk::SamplerCreateInfo<'_>) -> VkResult<vk::Sampler> {
        unsafe { self.handle().create_sampler(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::Sampler> for Device {
    fn destroy(&self, vk_struct: vk::Sampler) {
        unsafe {
            self.handle().destroy_sampler(vk_struct, None);
        }
    }
}
