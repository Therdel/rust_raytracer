use std::io::Cursor;

use nalgebra_glm as glm;
use wasm_bindgen::prelude::wasm_bindgen;

use lib_raytracer::Scene;
use lib_raytracer::scene_file::Parser;
use lib_raytracer::raytracing::raytracer::Raytracer;


use crate::utils;
use crate::color::{ColorRgbaU8, QuantizeToU8};
use crate::fake_same_mesh_loader::FakeSameMeshLoader;

#[wasm_bindgen]
pub struct Renderer {
    scene: Scene
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
        scene.resize_screen(width, height);

        Self {
            scene
        }
    }

    pub fn render(&self, canvas_u8: &mut [u8]) {
        let canvas = utils::canvas_from_raw_mut(canvas_u8);

        for y in 0..self.scene.screen().pixel_height {
            for x in 0..self.scene.screen().pixel_width {
                self.render_pixel(canvas, x, y);
            }
        }
    }

    pub fn render_interlaced(&self, canvas_u8: &mut [u8], y_offset: usize, row_jump: usize) {
        let canvas = utils::canvas_from_raw_mut(canvas_u8);

        for y in (y_offset..self.scene.screen().pixel_height).step_by(row_jump) {
            for x in 0..self.scene.screen().pixel_width {
                self.render_pixel(canvas, x, y);
            }
        }
    }

    fn render_pixel(&self, canvas: &mut [ColorRgbaU8], x: usize, y: usize) {
        let max_y_index = self.scene.screen().pixel_height - 1;
        let y_inverted = max_y_index - y;

        let coordinate = glm::vec2(x as _, y_inverted as _);
        let raytracer = Raytracer { scene: &self.scene };
        let ray = raytracer.generate_primary_ray(&coordinate);

        let color = match raytracer.raytrace(&ray) {
            Some(hit_color) => hit_color,
            None => self.scene.screen().background
        };
        let color = glm::vec4(color.x, color.y, color.z, 1.0);
        let offset = x + self.scene.screen().pixel_width * y;

        canvas[offset] = color.quantize();
    }

    pub fn resize_screen(&mut self, width: usize, height: usize) {
        self.scene.resize_screen(width, height)
    }

    pub fn turn_camera(&mut self, drag_begin_x: f32, drag_begin_y: f32, drag_end_x: f32, drag_end_y: f32) {
        self.scene.turn_camera(&glm::vec2(drag_begin_x, drag_begin_y),
                                          &glm::vec2(drag_end_x, drag_end_y));
    }
}
