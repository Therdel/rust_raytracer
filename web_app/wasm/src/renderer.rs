use std::io::Cursor;

use lib_raytracer::raytracing::{Background, Camera};
use lib_raytracer::raytracing::raytracer::Raytracer;
use lib_raytracer::Scene;
use lib_raytracer::scene_file::Parser;
use nalgebra_glm as glm;
use num_traits::zero;
use wasm_bindgen::prelude::wasm_bindgen;


use crate::asset_store::AssetStore;
use crate::color::{ColorRgbaU8, QuantizeToU8};
use crate::utils;

#[wasm_bindgen]
pub struct Renderer {
    scene: Scene
}

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32) -> Self {
        let camera = Camera {
            position: zero(),
            orientation: zero(),
            screen_dimensions: glm::vec2(width, height),
            y_fov_degrees: 90.,
            z_near: 0.1,
            z_far: 25.
        };
        let background = Background::ColoredDirection;
        let empty_scene = Scene::new(camera, background);

        Self { scene: empty_scene }
    }

    // TODO: handle failure when json interface cometh
    pub fn set_scene(&mut self, asset_store: &AssetStore, scene_file_name: &str) {
        // backup current screen dimensions, to restore them after the scene switch
        let width = self.scene.camera().screen_dimensions.x;
        let height = self.scene.camera().screen_dimensions.y;

        let Some(scene_buffer): Option<Vec<u8>> = asset_store.get_asset_bytes(scene_file_name) else {
            panic!("Loading scene '{scene_file_name}' was undefined")
        };

        let scene = Parser {
            file_reader: Cursor::new(scene_buffer),
            mesh_loader: asset_store,
        }.parse_json().unwrap();
        self.scene = scene;

        self.resize_screen(width as usize, height as usize)
    }

    pub fn render(&self, canvas_u8: &mut [u8]) {
        let canvas = utils::canvas_from_raw_mut(canvas_u8);

        for y in 0..self.scene.camera().screen_dimensions.y as usize {
            for x in 0..self.scene.camera().screen_dimensions.x as usize {
                self.render_pixel(canvas, x, y);
            }
        }
    }

    pub fn render_interlaced(&self, canvas_u8: &mut [u8], y_offset: usize, row_jump: usize) {
        let canvas = utils::canvas_from_raw_mut(canvas_u8);

        for y in (y_offset..self.scene.camera().screen_dimensions.y as usize).step_by(row_jump) {
            for x in 0..self.scene.camera().screen_dimensions.x as usize {
                self.render_pixel(canvas, x, y);
            }
        }
    }

    fn render_pixel(&self, canvas: &mut [ColorRgbaU8], x: usize, y: usize) {
        let max_y_index = self.scene.camera().screen_dimensions.y as usize - 1;
        let y_inverted = max_y_index - y;

        let coordinate = glm::vec2(x as _, y_inverted as _);
        let raytracer = Raytracer { scene: &self.scene };
        let ray = raytracer.generate_primary_ray(&coordinate);

        let color = match raytracer.raytrace(&ray) {
            Some(hit_color) => hit_color,
            None => raytracer.trace_background(&ray)
        };
        let color = glm::vec4(color.x, color.y, color.z, 1.0);
        let offset = x + self.scene.camera().screen_dimensions.x as usize  * y;

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
