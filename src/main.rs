mod exercise1;
mod raytracing;
mod utils;

use std::ffi::CString;
use crate::exercise1::{Canvas, Scene};
use crate::raytracing::{Triangle, Plane, Sphere, Light, Camera, Ray, Intersect};
use num_traits::AsPrimitive; // ::as_()

const IMAGE_PATH: &'static str = "render.png";
const IMAGE_WIDTH: usize = 1000;
const IMAGE_HEIGHT: usize = 1000;

fn main() -> std::io::Result<()> {
    let scene = test_scene();
    let screen_to_world = raytracing::transform::matrix::screen_to_world(&scene.camera);
    let mut canvas = Canvas::new((IMAGE_WIDTH, IMAGE_HEIGHT));

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let ray = generate_primary_ray(&glm::vec2(x.as_(), y.as_()),
                                           &screen_to_world);
            if let Some(hitpoint) = scene.intersect(&ray) {
                let scale = 1.0 / 10.0;
                let brightness = hitpoint.t * scale;
                let color = glm::vec3(brightness, brightness, brightness);
                canvas.set_pixel(x, y, &color);
            }
        }
    }
    canvas.write_png(CString::new(IMAGE_PATH)?.as_c_str());
    Ok(())
}

fn generate_primary_ray(screen_coordinate: &glm::Vec2, screen_to_world: &glm::Mat4) -> Ray {
    let p_screen = glm::vec4(screen_coordinate.x, screen_coordinate.y, 0.0, 1.0);
    // TODO: Document that NDC "looks" in *positive* z-axis. Document wrong viewing direction
    //       Erklärung: Hat was mit der z-Range zutun, wie man die definiert.
    // TODO: Document that this is *always* in camera view direction. (NDC)
    let p_screen_forward = p_screen + glm::vec4(0.0, 0.0, 1.0, 0.0);

    let p_world = *screen_to_world * p_screen;
    let p_world_forward = *screen_to_world * p_screen_forward;

    let p_world_inhomogeneous = (p_world / p_world.w).truncate(3);
    let p_world_forward_inhomogeneous = (p_world_forward / p_world_forward.w).truncate(3);

    let direction = p_world_forward_inhomogeneous - p_world_inhomogeneous;
    let direction_normalized = glm::normalize(direction);
    Ray {
        origin: p_world_inhomogeneous,
        direction: direction_normalized,
    }
}

fn test_scene() -> Scene {
    Scene {
        camera: Camera {
            position: glm::vec3(0.0, 0.0, -1.0),
            orientation: glm::vec3(0.0f32.to_radians(),
                                   0.0f32.to_radians(),
                                   0.0f32.to_radians()),
            pixel_width: 1000,
            pixel_height: 1000,
            y_fov_degrees: 90.0,
            z_near: 0.1, z_far: 25.0,
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
                normal: glm::vec3(0.0, -1.0, 0.0),
                distance: 5.0,
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