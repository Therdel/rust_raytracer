use crate::raytracing::color::ColorRgb;

pub struct Screen {
    pub pixel_width: usize,
    pub pixel_height: usize,

    pub background: ColorRgb
}