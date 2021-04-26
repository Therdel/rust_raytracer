pub type ColorRgb = glm::Vec3;
pub type ColorRgbU8 = [u8; 3];

pub trait Color {
    /// "Colors that end in 'urple'!"
    fn urple() -> Self;

    fn red() -> Self;
    fn green() -> Self;
    fn yellow() -> Self;
    fn black() -> Self;
}

impl Color for ColorRgb {
    fn urple() -> Self { glm::vec3(1.0, 0.5, 0.5) }

    fn red() -> Self { glm::vec3(1.0, 0.0, 0.0) }
    fn green() -> Self { glm::vec3(0.0, 1.0, 0.0) }
    fn yellow() -> Self { glm::vec3(1.0, 1.0, 0.0) }
    fn black() -> Self { glm::vec3(0.0, 0.0, 0.0) }
}

pub fn add_option(lhs: Option<ColorRgb>, rhs: Option<ColorRgb>) -> Option<ColorRgb> {
    if let Some(lhs) = lhs {
        Some(lhs + rhs?)
    } else {
        rhs
    }
}

pub trait QuantizeToU8 {
    fn quantize(&self) -> ColorRgbU8;
}

impl QuantizeToU8 for ColorRgb {
    fn quantize(&self) -> ColorRgbU8 {
        let mut clamped_color = glm::clamp(*self, glm::vec3(0., 0., 0.), glm::vec3(1., 1., 1.));
        clamped_color = clamped_color * 255.;

        [
            clamped_color.x as u8,
            clamped_color.y as u8,
            clamped_color.z as u8
        ]
    }
}