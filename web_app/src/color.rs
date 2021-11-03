use nalgebra_glm as glm;

pub type ColorRgba = glm::Vec4;
pub type ColorRgbaU8 = [u8; 4]; // TODO: replace with glm::U8Vec4?

pub trait QuantizeToU8 {
    fn quantize(&self) -> ColorRgbaU8;
}

impl QuantizeToU8 for ColorRgba {
    fn quantize(&self) -> ColorRgbaU8 {
        let mut clamped_color = glm::clamp(self, 0.0, 1.0);
        clamped_color = clamped_color * 255.;

        [
            clamped_color.x as u8,
            clamped_color.y as u8,
            clamped_color.z as u8,
            clamped_color.w as u8
        ]
    }
}