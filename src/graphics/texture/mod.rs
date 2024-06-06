use super::device::Device;
use ash::vk;
use std::ops::Deref;
use std::rc::Rc;

pub mod swapchain_image;

#[derive(Debug)]
pub struct Image {
    extent: vk::Extent3D,
    image: vk::Image,
    image_view: vk::ImageView,
    sampler: Option<vk::Sampler>,
    layer_count: u32,
    device: Rc<Device>,
}

impl Image {
    pub fn new() -> Self {
        todo!()
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
