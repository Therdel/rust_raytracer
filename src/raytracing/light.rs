use crate::utils::ColorRGB;

pub struct Light {
    pub position: glm::Vec4,
    pub color: LightColor,
}

pub struct LightColor {
    pub ambient: ColorRGB,
    pub diffuse: ColorRGB,
    pub specular: ColorRGB,
}