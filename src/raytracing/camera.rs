use nalgebra_glm as glm;

pub struct Camera {
    pub position: glm::Vec3,
    pub orientation: glm::Vec3,

    pub pixel_width: usize,
    pub pixel_height: usize,

    pub y_fov_degrees: f32,
    pub z_near: f32,
    pub z_far: f32,
}