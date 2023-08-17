use super::color::ColorRgb;

pub enum Background {
    SolidColor(ColorRgb),
    ColoredDirection,
    // HdrEnvironmentTexture,
}