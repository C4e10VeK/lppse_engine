use super::Image;
use crate::graphics::device::Device;
use ash::vk;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct SwapchainImage(Image);

impl SwapchainImage {
    pub fn new(
        device: Rc<Device>,
        image: vk::Image,
        format: vk::Format,
        extent: vk::Extent3D,
    ) -> Self {
        let image_view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .components(vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            })
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = device
            .create_image_view(&image_view_info)
            .expect("Error while create image view for swapchain image");

        let image_internal = Image {
            extent,
            image,
            image_view,
            sampler: None,
            layer_count: 1,
            device,
        };

        Self(image_internal)
    }
}

impl Deref for SwapchainImage {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for SwapchainImage {
    fn drop(&mut self) {
        if let Some(sampler) = self.sampler {
            self.device.destroy_sampler(sampler);
        }
        self.device.destroy_image_view(self.image_view);
    }
}
