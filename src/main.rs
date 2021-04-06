mod exercise1;
mod raytracing;
mod utils;

use std::ffi::CString;
use crate::exercise1::{Canvas, Scene};
use crate::raytracing::{Triangle, Plane, Sphere, Light, Camera};

const IMAGE_PATH: &'static str = "render.png";
const IMAGE_WIDTH: usize = 1000;
const IMAGE_HEIGHT: usize = 1000;

fn main() -> std::io::Result<()> {
    let _scene = test_scene();
    let mut canvas = Canvas::new((IMAGE_WIDTH, IMAGE_HEIGHT));

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let color = glm::vec3(1., 0.5, 0.5); // raytrace here
            canvas.set_pixel((x, y), &color);
        }
    }
    canvas.write_png(CString::new(IMAGE_PATH)?.as_c_str());
    Ok(())
}

fn test_scene() -> Scene {
    Scene {
        camera: Camera {
            position: glm::vec3(0.0, 0.0, 1.0),
            orientation: glm::vec3(0.0, 0.0, 0.0),
            pixel_width: 1000,
            pixel_height: 1000,
            fov: 90.0,
        },
        lights: vec![
            Light {
                position: glm::vec4(1.0, 1.0, 1.0, 0.0), // directional
                ambient: glm::vec3(0.1, 0.1, 0.1),
                diffuse: glm::vec3(0.2, 0.2, 0.2),
                specular: glm::vec3(0.5, 0.5, 0.5),
            }
        ],
        planes: vec![
            Plane {
                normal: glm::vec3(0.0, 0.0, -1.0),
                distance: 1.0,
            }
        ],
        spheres: vec![
            Sphere {
                center: glm::vec3(0.0, 0.0, -2.0),
                radius: 0.5,
            }
        ],
        triangles: vec![
            Triangle::new(
                glm::vec3(-5.0, -2.5, -5.0),
                glm::vec3(5.0, -2.5, -5.0),
                glm::vec3(0.0, 2.5, -5.0),
            )
        ],
    }
}