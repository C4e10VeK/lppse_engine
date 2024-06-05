pub mod gfx;

pub trait IntoExtent2D {
    fn into_extent(self) -> ash::vk::Extent2D;
}

impl<T: Into<u32>> IntoExtent2D for winit::dpi::PhysicalSize<T> {
    fn into_extent(self) -> ash::vk::Extent2D {
        let winit::dpi::PhysicalSize {
            width,
            height,
        } = self;
        
        ash::vk::Extent2D {
            width: width.into(),
            height: height.into(),
        }
    }
}

#[inline]
pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major) << 22) | ((minor) << 12) | (patch)
}
