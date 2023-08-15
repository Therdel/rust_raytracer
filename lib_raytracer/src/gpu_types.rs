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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Material {
    emissive: glm::Vec4,
    ambient: glm::Vec4,
    diffuse: glm::Vec4,
    specular: glm::Vec3,
    shininess: f32,

    material_type: u32,
    /// only set on material_type == ReflectAndRefract
    index_inner: f32,
    /// only set on material_type == ReflectAndRefract
    index_outer: f32,
    _padding: [u8; 4],
}

impl From<&raytracing::Material> for Material {
    fn from(value: &raytracing::Material) -> Self {
        let (mut index_inner, mut index_outer) = (0.0, 0.0);
        let material_type = match value.material_type {
            raytracing::MaterialType::Phong => MaterialType::Phong as _,
            raytracing::MaterialType::ReflectAndPhong => MaterialType::ReflectAndPhong as _,
            raytracing::MaterialType::ReflectAndRefract { index_inner: index_inner_enum, index_outer: index_outer_enum } => {
                index_inner = index_inner_enum;
                index_outer = index_outer_enum;
                MaterialType::ReflectAndRefract as _
            },
        };
        Self {
            emissive: glm::vec3_to_vec4(&value.emissive),
            ambient: glm::vec3_to_vec4(&value.ambient),
            diffuse: glm::vec3_to_vec4(&value.diffuse),
            specular: value.specular,
            shininess: value.shininess,
            material_type,
            index_inner,
            index_outer,
            _padding: Default::default(),
        }
    }
}

#[repr(u32)]
pub enum MaterialType {
    Phong = 0,
    ReflectAndPhong = 1,
    ReflectAndRefract = 2,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Light {
    position: glm::Vec4,
    color: LightColor,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightColor {
    ambient: glm::Vec4,
    diffuse: glm::Vec4,
    specular: glm::Vec4,
}

impl From<&raytracing::Light> for Light {
    fn from(value: &raytracing::Light) -> Self {
        Self {
            position: value.position,
            color: LightColor {
                ambient: glm::vec3_to_vec4(&value.color.ambient),
                diffuse: glm::vec3_to_vec4(&value.color.diffuse),
                specular: glm::vec3_to_vec4(&value.color.specular),
            }
        }
    }
}
