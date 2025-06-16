use lib_raytracer::{raytracing::color::ColorRgbU8, Canvas};
use std::{fs::File, io, path::Path};

pub trait WritePng {
    fn write_png(&self, path: &Path);
}

impl WritePng for Canvas {
    fn write_png(&self, path: &Path) {
        let file = File::create(path).unwrap();
        let file_writer = io::BufWriter::new(file);

        let mut png_encoder = png::Encoder::new(file_writer, self.get_width() as u32, self.get_height() as u32);
        png_encoder.set_color(png::ColorType::Rgb);
        let mut png_writer = png_encoder.write_header().unwrap();

        let pixels_raw = {
            let pixels = self.get_pixels();
            let raw_ptr = pixels.as_ptr() as *const u8;
            let pixels_byte_len = pixels.len() * std::mem::size_of::<ColorRgbU8>();
            unsafe {
                std::slice::from_raw_parts(raw_ptr, pixels_byte_len)
            }
        };

        png_writer.write_image_data(pixels_raw).unwrap();
    }
}