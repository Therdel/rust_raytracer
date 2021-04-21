use crate::raytracing::color::ColorRgb;

pub struct Light {
    pub position: glm::Vec4,
    pub color: LightColor,
}

pub struct LightColor {
    pub ambient: ColorRgb,
    pub diffuse: ColorRgb,
    pub specular: ColorRgb,
}