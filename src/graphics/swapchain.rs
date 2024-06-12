use super::device::{Device, DeviceCreateExtend, DeviceDestroyExtend};
use super::surface::Surface;
use super::synchronization::semaphore::Semaphore;
use super::texture::swapchain_image::SwapchainImage;
use crate::debug_log;
use crate::utils::IntoExtent3D;
use ash::prelude::VkResult;
use ash::vk;
use std::cell::RefCell;
use std::ops::{Add, Deref};
use std::rc::Rc;

#[derive(Debug)]
pub struct Swapchain {
    images: Vec<SwapchainImage>,
    handle: vk::SwapchainKHR,
    device: Rc<Device>,

    image_format: vk::Format,
    present_mode: vk::PresentModeKHR,
    extent: vk::Extent2D,
    current_image: u32,
}

impl Swapchain {
    pub fn new(
        device: Rc<Device>,
        surface: &Surface,
        description: SwapchainDescription,
    ) -> VkResult<Self> {
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

        let swapchain = device.create(&create_info)?;

        let extent = description.image_description.extent;
        let image_extent = extent.into_extent3d();

        let images: Vec<_> = device
            .get_swapchain_images(swapchain)
            .expect("Error while get swapchain images")
            .into_iter()
            .map(|image| {
                SwapchainImage::new(
                    device.clone(),
                    image,
                    description.image_description.format,
                    image_extent,
                )
            })
            .collect();

        Ok(Self {
            images,
            handle: swapchain,
            device,
            image_format: description.image_description.format,
            present_mode: description.present_mode,
            extent,
            current_image: 0,
        })
    }
    
    pub fn handle(&self) -> vk::SwapchainKHR {
        self.handle
    }

    pub fn get_current_image(&mut self, present_semaphore: &Semaphore) -> VkResult<(SwapchainImage, u32)> {
        let (index, _suboptimal) = unsafe {
            self.device.swapchain_fns().acquire_next_image(
                self.handle,
                u64::MAX,
                present_semaphore.handle(),
                vk::Fence::null(),
            )?
        };

        let image = self.images[index as usize].clone();
        self.current_image = index;

        Ok((image, index))
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        debug_log!(stringify!(Swapchain::drop()));
        for image in &self.images {
            image.destroy();
        }
        self.device.destroy(self.handle);
    }
}

impl DeviceCreateExtend<vk::SwapchainCreateInfoKHR<'_>, vk::SwapchainKHR> for Device {
    fn create(&self, create_info: &vk::SwapchainCreateInfoKHR<'_>) -> VkResult<vk::SwapchainKHR> {
        unsafe { self.swapchain_fns().create_swapchain(create_info, None) }
    }
}

impl DeviceDestroyExtend<vk::SwapchainKHR> for Device {
    fn destroy(&self, vk_struct: vk::SwapchainKHR) {
        unsafe {
            self.swapchain_fns().destroy_swapchain(vk_struct, None);
        }
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
