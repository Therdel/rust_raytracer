use crate::raytracing::color::ColorRgb;
use nalgebra_glm as glm;

pub struct Light {
    pub position: glm::Vec4,
    pub color: LightColor,
}

pub struct LightColor {
    pub ambient: ColorRgb,
    pub diffuse: ColorRgb,
    pub specular: ColorRgb,
}