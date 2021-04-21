use crate::raytracing::color::*;
use std::ffi::CStr;
use stb::image_write::stbi_write_png;


pub struct Canvas {
    pixels: Vec<ColorRgbU8>,
    width: usize,
    height: usize
}

impl Canvas {
    pub fn new(canvas_dimensions: (usize, usize), background: ColorRgb) -> Canvas {
        Canvas {
            pixels: vec![background.quantize(); canvas_dimensions.0 * canvas_dimensions.1],
            width: canvas_dimensions.0,
            height: canvas_dimensions.1
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: &ColorRgb) {
        let max_y_index = self.height - 1;
        let y_inverted = max_y_index - y;
        let pos = x + self.width * y_inverted;
        self.pixels[pos] = color.quantize();
    }

    pub fn write_png(&self, path: &CStr) {
        let channels = 3;
        let pixels_raw = unsafe {
            let raw_ptr = self.pixels.as_ptr() as *const u8;
            std::slice::from_raw_parts(raw_ptr, self.pixels.len() * channels)
        };
        let stride_in_bytes = 3 * self.width as i32;
        stbi_write_png(path, self.width as i32, self.height as i32, channels as i32, pixels_raw, stride_in_bytes);
    }
}