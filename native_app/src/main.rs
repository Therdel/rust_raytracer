use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use nalgebra_glm as glm;
use rayon::prelude::*;

mod write_png;
mod filesystem_mesh_loader;

use filesystem_mesh_loader::*;
use write_png::*;
use lib_raytracer::{Canvas, Scene};
use lib_raytracer::scene_file::Parser;
use lib_raytracer::raytracing::raytracer::Raytracer;

// TODO: Grab from args or default to stdout
const IMAGE_PATH: &str = "render.png";
const MODEL_DIR_PATH: &str = "res/models";
const SCENE_PATH: &str = "res/scenes/og_scene_rust.json";
// const SCENE_PATH: &str = "res/scenes/purple_marbles.json";
// const SCENE_PATH: &str = "res/scenes/cornell_box.json";
// const SCENE_PATH: &str = "res/scenes/santa_and_balls.json";

fn main() {
    let scene_file = BufReader::new(File::open(SCENE_PATH).unwrap());
    let mesh_loader = FilesystemMeshLoader {
        model_dir: MODEL_DIR_PATH.into()
    };

    let time_start = Instant::now();
    let scene = Parser {
        file_reader: scene_file,
        mesh_loader,
    }.parse_json().unwrap();
    println!("Parsing took {:.2}s", time_start.elapsed().as_secs_f32());

    let time_start = Instant::now();
    let canvas = paint_scene(scene);
    println!("Rendering took {:.2}s", time_start.elapsed().as_secs_f32());

    let path = CString::new(IMAGE_PATH)
        .unwrap_or_else(|_| panic!("Invalid target image path: ('{}')", IMAGE_PATH));
    canvas.write_png(path.as_c_str());
}

fn paint_scene(scene: Scene) -> Canvas {
    let mut canvas = Canvas::new(scene.camera());
    let pixel_width = scene.camera().screen_dimensions.x as _;
    let raytracer = Raytracer{ scene: &scene };

    canvas.borrow_stripes_mut()
        .par_bridge()
        .for_each(|mut row_stripe| {
            let y = row_stripe.get_y_coord();
            for x in 0..pixel_width {
                let coordinate = glm::vec2(x as _, y as _);
                let ray = raytracer.generate_primary_ray(&coordinate);
                let hit_color = match raytracer.raytrace(&ray) {
                    Some(hit_color) => hit_color,
                    None => raytracer.trace_background(&ray)
                };
                row_stripe.set_pixel(x, &hit_color);
            }
        });
    canvas
}