use nalgebra_glm as glm;
use crate::raytracing;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Camera {
    screen_to_world: glm::Mat4,
    screen_dimensions: glm::U32Vec2,
    _padding: [u32; 2],
}

impl From<&crate::Scene> for Camera {
    fn from(value: &crate::Scene) -> Self {
        Self {
            screen_to_world: *value.screen_to_world(),
            screen_dimensions: value.camera().screen_dimensions,
            _padding: Default::default()
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Background {
    /// only set on background_type == SolidColor
    solid_color: glm::Vec3,
    background_type: u32,
}

impl From<&raytracing::Background> for Background {
    fn from(value: &raytracing::Background) -> Self {
        match *value {
            raytracing::Background::SolidColor(color) => Self {
                solid_color: color,
                background_type: 0,
            },
            raytracing::Background::ColoredDirection => Self {
                solid_color: Default::default(),
                background_type: 1
            }
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// Fixed-size array to reduce amount of needed storage buffers
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct PlanesAndTriangles {
    planes: [Plane; PlanesAndTriangles::LEN],
    triangles: [Triangle; PlanesAndTriangles::LEN],
    planes_len: u32,
    triangles_len: u32,

    /// array stride padding - make size a multiple of 16 to correctly align the vec3 elements
    /// ([source](https://stackoverflow.com/a/75525055))
    _padding: [u32; 2],
}

impl PlanesAndTriangles {
    const LEN: usize = 64;
}

impl<'a> TryFrom<(&'a[raytracing::Plane], &'a[raytracing::Triangle])> for PlanesAndTriangles {
    type Error = &'static str;

    fn try_from(value: (&'a[raytracing::Plane], &'a[raytracing::Triangle])) -> Result<Self, Self::Error> {
        let (planes, triangles) = value;
        if planes.len() > PlanesAndTriangles::LEN { return Err("Too much planes in scene for GPU limit") }
        if triangles.len() > PlanesAndTriangles::LEN { return Err("Too much triangles in scene for GPU limit") }

        let mut planes_array = [Plane::empty(); PlanesAndTriangles::LEN];
        let mut triangle_array = [Triangle::empty(); PlanesAndTriangles::LEN];

        for (gpu_plane, plane) in planes_array.iter_mut().zip(planes.iter()) {
            *gpu_plane = plane.into()
        }
        for (gpu_triangle, triangle) in triangle_array.iter_mut().zip(triangles.iter()) {
            *gpu_triangle = triangle.into()
        }

        Ok(Self {
            planes: planes_array,
            triangles: triangle_array,
            planes_len: planes.len() as _,
            triangles_len: triangles.len() as _,
            _padding: Default::default(),
        })
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Plane {
    normal: glm::Vec3,
    distance: f32,
    material: u32,
    /// array stride padding - make size a multiple of 16 to correctly align the vec3 elements
    /// ([source](https://stackoverflow.com/a/75525055))
    _padding: [u32; 3],
}

impl Plane {
    /// an invalid, empty value for use in e.g. empty array slots
    fn empty() -> Self {
        Self {
            normal: Default::default(),
            distance: Default::default(),
            material: Default::default(),
            _padding: Default::default(),
        }
    }
}

impl From<&raytracing::Plane> for Plane {
    fn from(value: &raytracing::Plane) -> Self {
        Self {
            normal: value.normal,
            distance: value.distance,
            material: value.material.0 as _,
            _padding: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Sphere {
    center: glm::Vec3,
    radius: f32,
    material: u32,
    /// array stride padding - make size a multiple of 16 to correctly align the vec3 elements
    /// ([source](https://stackoverflow.com/a/75525055))
    _padding: [u32; 3],
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
    _padding: [u32; 1],
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Triangle {
    vertices: [glm::Vec4; 3],
    normals: [glm::Vec4; 3],
    normal: glm::Vec3,

    material: u32,
}

impl Triangle {
    /// an invalid, empty value for use in e.g. empty array slots
    fn empty() -> Self {
        Self {
            vertices: Default::default(),
            normals: Default::default(),
            normal: Default::default(),
            material: Default::default(),
        }
    }
}

impl From<&raytracing::Triangle> for Triangle {
    fn from(value: &raytracing::Triangle) -> Self {
        Self {
            vertices: value.vertices.map(|v| glm::vec3_to_vec4(&v)),
            normals: value.normals.map(|v| glm::vec3_to_vec4(&v)),
            normal: *value.normal(),
            material: value.material.0 as _,
        }
    }
}

impl From<&raytracing::MeshTriangle> for Triangle {
    fn from(value: &raytracing::MeshTriangle) -> Self {
        Self {
            vertices: value.0.vertices.map(|v| glm::vec3_to_vec4(&v)),
            normals: value.0.normals.map(|v| glm::vec3_to_vec4(&v)),
            normal: *value.0.normal(),
            material: value.0.material.0 as _,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct BvhNode {
    aabb_min: glm::Vec4,
    aabb_max: glm::Vec3,
    is_leaf: u32,
    child_left_index: u32,
    child_right_index: u32,
    triangle_indices: [u32; raytracing::bvh::Node::LEAF_TRIANGLES],
    triangle_indices_len: u32,
}

impl From<&raytracing::bvh::Node> for BvhNode {
    fn from(value: &raytracing::bvh::Node) -> Self {
        let is_leaf;
        let child_left_index;
        let child_right_index;
        let triangle_indices: [u32; raytracing::bvh::Node::LEAF_TRIANGLES];
        let triangle_indices_len;

        match &value.content {
            &raytracing::bvh::NodeType::Node { child_left: value_child_left, child_right: value_child_right } => {
                is_leaf = 0;
                child_left_index = value_child_left as _;
                child_right_index = value_child_right as _;
                triangle_indices = Default::default();
                triangle_indices_len = Default::default();
            },
            raytracing::bvh::NodeType::Leaf { triangle_indices: value_triangle_indices } => {
                is_leaf = 1;
                child_left_index = Default::default();
                child_right_index = Default::default();

                let mut triangle_indices_mut = [0u32; raytracing::bvh::Node::LEAF_TRIANGLES];
                triangle_indices_mut.iter_mut().zip(value_triangle_indices.iter())
                    .for_each(|(lhs, rhs)| *lhs = *rhs as _);
                triangle_indices = triangle_indices_mut;
                triangle_indices_len = 30;
            },
        }
        Self {
            aabb_min: glm::vec3_to_vec4(&value.aabb.min),
            aabb_max: value.aabb.max,
            is_leaf,
            child_left_index,
            child_right_index,
            triangle_indices,
            triangle_indices_len,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct Mesh {
    triangle_indices_start: u32,
    triangle_indices_end: u32,
    bvh_node_indices_start: u32,
    bvh_node_indices_end: u32,
    bvh_max_depth: u32
}

impl From<&raytracing::Mesh> for Mesh {
    fn from(value: &raytracing::Mesh) -> Self {
        Self {
            triangle_indices_start: value.triangle_indices.start as _,
            triangle_indices_end: value.triangle_indices.end as _,
            bvh_node_indices_start: value.bvh.bvh_node_indices.start as _,
            bvh_node_indices_end: value.bvh.bvh_node_indices.end as _,
            bvh_max_depth: value.bvh.max_depth as _,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
/// WebGPU version with altered alignment & padding. ([source](https://stackoverflow.com/a/75525055))
pub struct MeshInstance {
    rotation_scale: glm::Mat4,
    rotation_scale_inverse: glm::Mat4,
    model: glm::Mat4,
    model_inverse: glm::Mat4,
    mesh_index: u32,
    material_override: u32,
    material_override_is_some: u32,
    _padding: [u32; 1],
}

impl From<&raytracing::Instance<raytracing::Mesh>> for MeshInstance {
    fn from(value: &raytracing::Instance<raytracing::Mesh>) -> Self {
        let (material_override, material_override_is_some) =
            match value.material_override {
                Some(material) => (material.0 as _, 1),
                None => (0, 0)
            };
        Self {
            mesh_index: value.primitive_index as _,
            rotation_scale: value.rotation_scale,
            rotation_scale_inverse: value.rotation_scale_inverse,
            model: value.model,
            model_inverse: value.model_inverse,
            material_override,
            material_override_is_some,
            _padding: Default::default(),
        }
    }
}