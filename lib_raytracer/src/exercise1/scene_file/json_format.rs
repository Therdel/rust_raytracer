use serde::{Deserialize};

#[derive(Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Deserialize)]
pub struct ColorRgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

#[derive(Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub screen: Screen,
    pub lights: Vec<Light>,
    pub materials: Vec<Material>,
    pub planes: Option<Vec<Plane>>,
    pub spheres: Option<Vec<Sphere>>,
    pub triangles: Option<Vec<Triangle>>,
    pub meshes: Option<Vec<Mesh>>,
    pub mesh_instances: Option<Vec<MeshInstance>>
}

#[derive(Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub orientation_degrees: Vec3,
    pub y_fov_degrees: f32,
    pub z_near: f32,
    pub z_far: f32,
}

#[derive(Deserialize)]
pub struct Screen {
    pub pixel_width: usize,
    pub pixel_height: usize,
    pub background: ColorRgb
}

#[derive(Deserialize)]
pub struct Light {
    pub position: Vec4,
    pub color: LightColor
}

#[derive(Deserialize)]
pub struct LightColor {
    pub ambient: ColorRgb,
    pub diffuse: ColorRgb,
    pub specular: ColorRgb
}


#[derive(Deserialize)]
pub struct Material {
    pub name: String,

    pub emissive: ColorRgb,
    pub ambient: ColorRgb,
    pub diffuse: ColorRgb,
    pub specular: ColorRgb,
    pub shininess: f32,

    pub material_type: MaterialType,
}


#[derive(Deserialize)]
pub enum MaterialType {
    Phong,
    ReflectAndPhong,
    ReflectAndRefract {
        index_inner: f32,
        index_outer: f32,
    }
}

#[derive(Deserialize)]
pub struct Plane {
    pub normal: Vec3,
    pub distance: f32,

    pub material: String,
}

#[derive(Deserialize)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,

    pub material: String,
}

#[derive(Deserialize)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub normals: [Vec3; 3],

    pub material: String,
}

#[derive(Deserialize)]
pub struct Mesh {
    pub name: String,
    pub file_name: String
}

#[derive(Deserialize)]
pub struct MeshInstance {
    pub mesh: String,
    pub position: Vec3,
    pub orientation_degrees: Vec3,
    pub scale: Vec3,
    pub material_override: Option<String>
}