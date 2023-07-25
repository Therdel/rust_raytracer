use lib_raytracer::Canvas;
use std::ffi::CStr;
use stb::image_write::stbi_write_png;

pub trait WritePng {
    fn write_png(&self, path: &CStr);
}

impl WritePng for Canvas {
    fn write_png(&self, path: &CStr) {
        let channels = 3;
        let pixels_raw = unsafe {
            let pixels = self.get_pixels();
            let raw_ptr = pixels.as_ptr() as *const u8;
            std::slice::from_raw_parts(raw_ptr, pixels.len() * channels)
        };
        let stride_in_bytes = (channels * self.get_width()) as i32;
        stbi_write_png(path,
                       self.get_width() as i32,
                       self.get_height() as i32,
                       channels as i32,
                       pixels_raw,
                       stride_in_bytes);
    }
}