use std::ffi::CStr;
use stb::image_write::stbi_write_png;

type ColorRgb = [u8; 3];

pub struct Canvas {
    pixels: Vec<ColorRgb>,
    width: usize,
    height: usize
}

impl Canvas {
    pub fn new(canvas_dimensions: (usize, usize)) -> Canvas {
        const URPLE: ColorRgb = [255, 127, 127];
        Canvas {
            pixels: vec![URPLE; canvas_dimensions.0 * canvas_dimensions.1],
            width: canvas_dimensions.0,
            height: canvas_dimensions.1
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: &glm::Vec3) {
        let max_y_index = self.height - 1;
        let y_inverted = max_y_index - y;
        let pos = x + self.width * y_inverted;
        self.pixels[pos] = Self::quantize_colors(color);
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

impl Canvas {
    fn quantize_colors(color: &glm::Vec3) -> ColorRgb {
        let mut clamped_color = glm::clamp(*color, glm::vec3(0., 0., 0.), glm::vec3(1., 1., 1.));
        clamped_color = clamped_color * 255.;

        [
            clamped_color.x as u8,
            clamped_color.y as u8,
            clamped_color.z as u8
        ]
    }
}