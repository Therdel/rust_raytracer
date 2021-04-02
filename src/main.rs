mod exercise1;
mod raytracing;
mod utils;
pub use utils::*;

use stb::image_write::stbi_write_png;
use std::ffi::CString;
use std::ops::Mul;

const IMAGE_PATH: &'static str = "render.png";
const IMAGE_WIDTH: usize = 1000;
const IMAGE_HEIGHT: usize = 1000;

fn main() -> std::io::Result<()> {
    let mut pixels = vec![[0u8, 0u8, 0u8]; IMAGE_WIDTH * IMAGE_HEIGHT];
    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let color = glm::vec3(50., 100., 100.); // raytrace here
            pixels[x + y*IMAGE_WIDTH] = quantize_colors(&color);
        }
    }

    let channels = 3;
    let stride_in_bytes = 3 * IMAGE_WIDTH as i32;
    let path_c_string = CString::new(IMAGE_PATH)?;
    let pixels_raw = unsafe {
        let raw_ptr = pixels.as_ptr() as *const u8;
        std::slice::from_raw_parts(raw_ptr, pixels.len() * channels)
    };
    stbi_write_png(path_c_string.as_c_str(), IMAGE_WIDTH as i32, IMAGE_HEIGHT as i32, channels as i32, pixels_raw, stride_in_bytes);

    Ok(())
}

fn quantize_colors(color: &glm::Vec3) -> [u8;3] {
    let mut clamped_color = glm::clamp(*color, glm::vec3(0., 0., 0.), glm::vec3(1., 1., 1.));
    clamped_color = clamped_color.mul(255.);

    [
        clamped_color.x as u8,
        clamped_color.y as u8,
        clamped_color.z as u8
    ]
}
