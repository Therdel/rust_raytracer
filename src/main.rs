mod exercise1;
mod raytracing;
mod utils;

use std::ffi::CString;
use exercise1::Canvas;

const IMAGE_PATH: &'static str = "render.png";
const IMAGE_WIDTH: usize = 1000;
const IMAGE_HEIGHT: usize = 1000;

fn main() -> std::io::Result<()> {
    let mut canvas = Canvas::new((IMAGE_WIDTH, IMAGE_HEIGHT));

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let color = glm::vec3(0.5, 0.5, 0.5); // raytrace here
            canvas.set_pixel((x, y), &color);
        }
    }
    canvas.write_png(CString::new(IMAGE_PATH)?.as_c_str());
    Ok(())
}