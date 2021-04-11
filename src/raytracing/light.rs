type Color = glm::Vec3;

pub struct Light {
    pub position: glm::Vec4,
    pub color: LightColor,
}

pub struct LightColor {
    pub ambient: Color,
    pub diffuse: Color,
    pub specular: Color,
}