use std::io::Cursor;
use std::slice;

use nalgebra_glm as glm;
use wasm_bindgen::prelude::*;

use lib_raytracer::exercise1::scene_file::Parser;
use lib_raytracer::raytracing::{color::*, raytracer::{Public, Raytracer}, Screen};

use crate::color::{ColorRgbaU8, QuantizeToU8};
use crate::fake_same_mesh_loader::FakeSameMeshLoader;

mod color;
mod fake_same_mesh_loader;
mod utils;

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub struct Renderer {
    width: usize,
    height: usize,
    raytracer: Raytracer,
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize,
               scene: &[u8], mesh_obj: &[u8]) -> Self {
        let mut scene = Parser {
            file_reader: Cursor::new(scene),
            mesh_loader: FakeSameMeshLoader { mesh_obj },
        }.parse_json().unwrap();
        scene.screen = Screen {
            pixel_width: width,
            pixel_height: height,
            background: Color::urple(),
        };
        let raytracer = Raytracer::new(scene);

        Renderer {
            width,
            height,
            raytracer,
        }
    }

    pub fn render(&self, canvas_u8: &mut [u8]) {
        let canvas_raw_color = canvas_u8.as_mut_ptr() as *mut ColorRgbaU8;
        let canvas = unsafe { slice::from_raw_parts_mut(canvas_raw_color, self.width * self.height) };

        for y in 0..self.height {
            for x in 0..self.width {
                let coordinate = glm::vec2(x as _, y as _);
                let ray = self.raytracer.generate_primary_ray(&coordinate);

                let color = match self.raytracer.raytrace(&ray) {
                    Some(hit_color) => hit_color,
                    None => Color::urple(), //scene.screen.background
                };
                let color = glm::vec4(color.x, color.y, color.z, 1.0);

                let max_y_index = self.height - 1;
                let y_inverted = max_y_index - y;
                let offset = x + self.width * y_inverted;

                canvas[offset] = color.quantize();
            }
        }
    }
}