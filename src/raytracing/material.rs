use crate::raytracing::color::ColorRgb;

pub struct Material {
    pub name: String,

    pub emissive: ColorRgb,
    pub ambient: ColorRgb,
    pub diffuse: ColorRgb,
    pub specular: ColorRgb,
    pub shininess: f32,
}