pub struct Camera {
    pub position: glm::Vec3,
    pub orientation: glm::Vec3,

    pub pixel_width: usize,
    pub pixel_height: usize,

    pub fov: f32
}