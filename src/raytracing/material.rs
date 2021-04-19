use crate::utils::ColorRGB;

pub struct Material {
    pub name: String,

    pub emissive: ColorRGB,
    pub ambient: ColorRGB,
    pub diffuse: ColorRGB,
    pub specular: ColorRGB,
    pub shininess: f32,
}