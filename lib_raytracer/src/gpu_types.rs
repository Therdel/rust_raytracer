use nalgebra_glm as glm;
use crate::raytracing;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Sphere {
    center: glm::Vec3,
    radius: f32,
    material: u32,
    /// array stride padding - make size a multiple of 16 to correctly align the vec3 elements
    /// ([source](https://stackoverflow.com/a/75525055))
    _padding: [u8; 12],
}

impl From<&raytracing::Sphere> for Sphere {
    fn from(sphere: &raytracing::Sphere) -> Self {
        Self {
            center: sphere.center,
            radius: sphere.radius,
            material: sphere.material.0 as _,
            _padding: Default::default(),
        }
    }
}
