use nalgebra_glm::{Mat4, Vec4};
use std::mem::size_of;

#[derive(Copy, Clone)]
#[repr(C)]
pub(crate) struct PushConstants {
    pub mat0: Mat4,
    pub vec0: Vec4,
    pub vec1: Vec4,
    pub vec2: Vec4,
    pub vec3: Vec4,
}

impl PushConstants {
    pub(crate) fn as_bytes(&self) -> Vec<u8> {
        let size_in_bytes = size_of::<Self>();
        let u8_ptr = self as *const Self as *const u8;
        unsafe { std::slice::from_raw_parts(u8_ptr, size_in_bytes).to_vec() }
    }
}
