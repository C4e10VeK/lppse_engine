pub mod gfx;
pub mod macros;

pub trait IntoExtent2D {
    fn into_extent(self) -> ash::vk::Extent2D;
}

impl<T: Into<u32>> IntoExtent2D for winit::dpi::PhysicalSize<T> {
    fn into_extent(self) -> ash::vk::Extent2D {
        let winit::dpi::PhysicalSize { width, height } = self;

        ash::vk::Extent2D {
            width: width.into(),
            height: height.into(),
        }
    }
}

pub trait IntoExtent3D {
    fn into_extent3d(self) -> ash::vk::Extent3D;
}

impl IntoExtent3D for ash::vk::Extent2D {
    fn into_extent3d(self) -> ash::vk::Extent3D {
        let ash::vk::Extent2D { width, height } = self;

        ash::vk::Extent3D {
            width,
            height,
            depth: 0,
        }
    }
}

#[inline]
pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major) << 22) | ((minor) << 12) | (patch)
}
