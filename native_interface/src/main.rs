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
use lib_raytracer::exercise1::{Canvas, Scene};
use lib_raytracer::exercise1::scene_file::Parser;
use lib_raytracer::raytracing::raytracer::{Public, Raytracer};

// TODO: Grab from args or default to stdout
const IMAGE_PATH: &'static str = "render.png";
const SCENE_PATH: &'static str = "res/scenes/scene_rust.json";
const MODEL_DIR_PATH: &'static str = "res/models";

fn main() {
    let scene_file = BufReader::new(File::open(SCENE_PATH).unwrap());
    let mesh_loader = FilesystemMeshLoader {
        model_dir: MODEL_DIR_PATH.into()
    };

    let scene = Parser {
        file_reader: scene_file,
        mesh_loader,
    }.parse_json().unwrap();

    let time_start = Instant::now();
    let canvas = paint_scene(&scene);
    println!("Rendering took {:.2}s", time_start.elapsed().as_secs_f32());

    let path = CString::new(IMAGE_PATH)
        .expect(&format!("Invalid target image path: ('{}')", IMAGE_PATH));
    canvas.write_png(path.as_c_str());
}

fn paint_scene(scene: &Scene) -> Canvas {
    let raytracer = Raytracer::new(&scene);
    let mut canvas = Canvas::new(&scene.screen);

    canvas.borrow_stripes_mut()
        .par_bridge()
        .for_each(|mut row_stripe| {
            let y = row_stripe.get_y_coord();
            for x in 0..scene.screen.pixel_width {
                let coordinate = glm::vec2(x as _, y as _);
                let ray = raytracer.generate_primary_ray(&coordinate);
                if let Some(hit_color) = raytracer.raytrace(&ray) {
                    row_stripe.set_pixel(x, &hit_color);
                }
            }
        });
    canvas
}