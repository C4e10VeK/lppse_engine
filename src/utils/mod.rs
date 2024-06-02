pub mod gfx;

#[inline]
pub const fn make_version(major: u32, minor: u32, patch: u32) -> u32 {
    ((major) << 22) | ((minor) << 12) | (patch)
}
