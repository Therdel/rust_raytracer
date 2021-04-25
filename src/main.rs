mod exercise1;
mod raytracing;
mod utils;

use std::ffi::CString;
use crate::exercise1::{Canvas, Scene};
use crate::raytracing::{
    Triangle, Plane, Sphere, Light, Camera, LightColor, Material, color::*,
    raytracer::{Raytracer, Public}
};

const IMAGE_PATH: &'static str = "render.png";
const IMAGE_WIDTH: usize = 1000;
const IMAGE_HEIGHT: usize = 1000;

fn main() -> std::io::Result<()> {
    let mut scene = Scene {
        camera: test_camera(IMAGE_WIDTH, IMAGE_HEIGHT),
        lights: test_lights(),
        planes: vec![],
        spheres: vec![],
        triangles: vec![],
        materials: vec![]
    };
    scene.materials = test_materials();
    scene.planes = test_planes(&scene.materials);
    scene.spheres = test_spheres(&scene.materials);
    scene.triangles = test_triangles(&scene.materials);

    let mut canvas = Canvas::new((IMAGE_WIDTH, IMAGE_HEIGHT), Color::black());
    let raytracer = Raytracer::new(&scene);

    for y in 0..IMAGE_HEIGHT {
        for x in 0..IMAGE_WIDTH {
            let coordinate = glm::vec2(x as _, y as _);
            let ray = raytracer.generate_primary_ray(&coordinate);
            if let Some(hit_color) = raytracer.raytrace(&ray) {
                canvas.set_pixel(x, y, &hit_color);
            }
        }
    }
    canvas.write_png(CString::new(IMAGE_PATH)?.as_c_str());
    Ok(())
}

fn test_lights() -> Vec<Light> {
    vec![
        Light {
            position: glm::vec4(1.0, 5.0, 1.0, 1.0), // directional
            color: LightColor {
                ambient: glm::vec3(0.1, 0.1, 0.1),
                diffuse: glm::vec3(0.5, 0.5, 0.5),
                specular: glm::vec3(0.5, 0.5, 0.5),
            }
        }
    ]
}

fn test_camera(width: usize, height: usize) -> Camera {
    Camera {
        position: glm::vec3(0.0, 0.0, -1.0),
        orientation: glm::vec3(0.0f32.to_radians(),
                               20.0f32.to_radians(),
                               0.0f32.to_radians()),
        pixel_width: width,
        pixel_height: height,
        y_fov_degrees: 90.0,
        z_near: 0.1, z_far: 25.0,
    }
}

fn test_materials() -> Vec<Material> {
    vec![
        Material {
            name: String::from("some_shiny_red"),
            emissive: glm::vec3(0.1, 0.0, 0.0),
            ambient: glm::vec3(0.4, 0.0, 0.0),
            diffuse: glm::vec3(0.4, 0.0, 0.0),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0
        },
        Material {
            name: String::from("some_shiny_yellow"),
            emissive: glm::vec3(0.1, 0.1, 0.0),
            ambient: glm::vec3(0.4, 0.4, 0.0),
            diffuse: glm::vec3(0.4, 0.4, 0.0),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0
        },
        Material {
            name: String::from("some_shiny_green"),
            emissive: glm::vec3(0.0, 0.1, 0.0),
            ambient: glm::vec3(0.0, 0.4, 0.0),
            diffuse: glm::vec3(0.0, 0.4, 0.0),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0
        },
    ]
}

fn test_triangles(materials: &[Material]) -> Vec<Triangle> {
    vec![
        Triangle::new(
            glm::vec3(-5.0, -2.5, -5.0),
            glm::vec3(5.0, -2.5, -5.0),
            glm::vec3(0.0, 2.5, -5.0),
            materials.iter().find(|&material| {
                material.name == "some_shiny_yellow"
            }).unwrap()
        )
    ]
}

fn test_planes(materials: &[Material]) -> Vec<Plane> {
    vec![
        Plane {
            normal: glm::vec3(0.0, -1.0, 0.0),
            distance: 5.0,
            material: materials.iter().find(|&material| {
                material.name == "some_shiny_green"
            }).unwrap()
        }
    ]
}

fn test_spheres(materials: &[Material]) -> Vec<Sphere> {
    vec![
        Sphere {
            center: glm::vec3(0.0, 1.0, -5.0),
            radius: 0.5,
            material: materials.iter().find(|&material| {
                material.name == "some_shiny_red"
            }).unwrap()
        },
        Sphere {
            center: glm::vec3(0.0, 0.0, -4.0),
            radius: 0.5,
            material: materials.iter().find(|&material| {
                material.name == "some_shiny_red"
            }).unwrap()
        },
        Sphere {
            center: glm::vec3(0.0, -1.0, -3.0),
            radius: 0.5,
            material: materials.iter().find(|&material| {
                material.name == "some_shiny_red"
            }).unwrap()
        }
    ]
}

// TODO: Can I ever return this?
// fn test_scene() -> Box<Scene<'static>> {
//     let materials = vec![
//         Material {
//             name: String::from("some_shiny_red"),
//             emissive: glm::vec3(0.1, 0.0, 0.0),
//             ambient: glm::vec3(0.4, 0.0, 0.0),
//             diffuse: glm::vec3(0.4, 0.0, 0.0),
//             specular: glm::vec3(0.6, 0.6, 0.6),
//             shininess: 10.0
//         },
//         Material {
//             name: String::from("some_shiny_yellow"),
//             emissive: glm::vec3(0.1, 0.1, 0.0),
//             ambient: glm::vec3(0.4, 0.4, 0.0),
//             diffuse: glm::vec3(0.4, 0.4, 0.0),
//             specular: glm::vec3(0.6, 0.6, 0.6),
//             shininess: 10.0
//         },
//         Material {
//             name: String::from("some_shiny_green"),
//             emissive: glm::vec3(0.0, 0.1, 0.0),
//             ambient: glm::vec3(0.0, 0.4, 0.0),
//             diffuse: glm::vec3(0.0, 0.4, 0.0),
//             specular: glm::vec3(0.6, 0.6, 0.6),
//             shininess: 10.0
//         },
//     ];
//
//     let scene = Box::from(Scene {
//         camera: Camera {
//             position: glm::vec3(0.0, 0.0, -1.0),
//             orientation: glm::vec3(0.0f32.to_radians(),
//                                    0.0f32.to_radians(),
//                                    0.0f32.to_radians()),
//             pixel_width: 1000,
//             pixel_height: 1000,
//             y_fov_degrees: 90.0,
//             z_near: 0.1, z_far: 25.0,
//         },
//         lights: vec![
//             Light {
//                 position: glm::vec4(1.0, 1.0, 1.0, 0.0), // directional
//                 color: LightColor {
//                     ambient: glm::vec3(0.1, 0.1, 0.1),
//                     diffuse: glm::vec3(0.2, 0.2, 0.2),
//                     specular: glm::vec3(0.5, 0.5, 0.5),
//                 }
//             }
//         ],
//         planes: vec![],
//         spheres: vec![],
//         triangles: vec![],
//         materials: materials
//     });
//
//     scene.planes = vec![
//         Plane {
//             normal: glm::vec3(0.0, -1.0, 0.0),
//             distance: 5.0,
//             material: scene.get_material("some_shiny_red").unwrap()
//         }
//     ];
//     scene.spheres = vec![
//         Sphere {
//             center: glm::vec3(0.0, 0.0, -2.0),
//             radius: 0.5,
//             material: scene.get_material("some_shiny_yellow").unwrap()
//         }
//     ];
//     scene.triangles = vec![
//         Triangle::new(
//             glm::vec3(-5.0, -2.5, -5.0),
//             glm::vec3(5.0, -2.5, -5.0),
//             glm::vec3(0.0, 2.5, -5.0),
//             scene.get_material("some_shiny_red").unwrap()
//         )
//     ];
//
//     scene
// }

// scene.triangles = vec![
//     Triangle::new(
//         glm::vec3(-5.0, -2.5, -5.0),
//         glm::vec3(5.0, -2.5, -5.0),
//         glm::vec3(0.0, 2.5, -5.0),
//         scene.materials.iter().find(|&material| {
//             material.name == "some_shiny_red"
//         }).unwrap()
//     )
// ];