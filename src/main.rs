mod exercise1;
mod raytracing;
mod utils;

use std::ffi::CString;
use crate::exercise1::{Canvas, Scene};
use crate::raytracing::{
    Triangle, Plane, Sphere, Light, Camera, LightColor, Material, MaterialType, color::*, Instance,
    raytracer::{Raytracer, Public},
};
use rayon::prelude::*;
use std::time::Instant;
use num_traits::zero;

const IMAGE_PATH: &'static str = "render.png";

fn main() {
    let mut scene = Scene {
        camera: test_camera(3640, 2160),
        background: Color::urple(),
        lights: test_lights(),
        planes: vec![],
        spheres: vec![],
        triangles: vec![],
        materials: vec![],
        instanced_spheres: vec![]
    };
    scene.materials = test_materials();
    scene.planes = test_planes(&scene.materials);
    scene.spheres = test_spheres(&scene.materials);
    scene.triangles = test_triangles(&scene.materials);
    scene.instanced_spheres = test_instanced_spheres(&scene.materials, &scene.spheres);

    let time_start = Instant::now();
    let canvas = paint_scene(&scene);
    println!("Rendering took {:.2}s", time_start.elapsed().as_secs_f32());

    let path = CString::new(IMAGE_PATH)
        .expect(&format!("Invalid target image path: ('{}')", IMAGE_PATH));
    canvas.write_png(path.as_c_str())
}

fn paint_scene(scene: &Scene) -> Canvas {
    let raytracer = Raytracer::new(&scene);
    let canvas_dimensions = (scene.camera.pixel_width, scene.camera.pixel_height);
    let mut canvas = Canvas::new(canvas_dimensions, scene.background);

    canvas.borrow_stripes_mut()
        .par_bridge()
        .for_each(|mut row_stripe| {
            let y = row_stripe.get_y_coord();
            for x in 0..scene.camera.pixel_width {
                let coordinate = glm::vec2(x as _, y as _);
                let ray = raytracer.generate_primary_ray(&coordinate);
                if let Some(hit_color) = raytracer.raytrace(&ray) {
                    row_stripe.set_pixel(x, &hit_color);
                }
            }
        });
    canvas
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
        position: glm::vec3(3.0, 0.0, 1.0),
        orientation: glm::vec3(0.0f32.to_radians(),
                               25.0f32.to_radians(),
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
            shininess: 10.0,
            material_type: MaterialType::Phong,
        },
        Material {
            name: String::from("some_shiny_yellow"),
            emissive: glm::vec3(0.1, 0.1, 0.0),
            ambient: glm::vec3(0.4, 0.4, 0.0),
            diffuse: glm::vec3(0.4, 0.4, 0.0),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0,
            material_type: MaterialType::Phong,
        },
        Material {
            name: String::from("some_shiny_green"),
            emissive: glm::vec3(0.0, 0.1, 0.0),
            ambient: glm::vec3(0.0, 0.4, 0.0),
            diffuse: glm::vec3(0.0, 0.4, 0.0),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0,
            material_type: MaterialType::Phong,
        },
        Material {
            name: String::from("some_shiny_white"),
            emissive: glm::vec3(0.1, 0.1, 0.1),
            ambient: glm::vec3(0.4, 0.4, 0.4),
            diffuse: glm::vec3(0.4, 0.4, 0.4),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0,
            material_type: MaterialType::Phong,
        },
        Material {
            name: String::from("some_shiny_blue"),
            emissive: glm::vec3(0.0, 0.037, 0.072),
            ambient: glm::vec3(0.0, 0.148, 0.288),
            diffuse: glm::vec3(0.0, 0.148, 0.288),
            specular: glm::vec3(0.6, 0.6, 0.6),
            shininess: 10.0,
            material_type: MaterialType::Phong,
        },
        Material {
            name: String::from("reflective"),
            emissive: glm::vec3(0.0, 0.0, 0.0),
            ambient: glm::vec3(0.0, 0.0, 0.0),
            diffuse: glm::vec3(0.0, 0.0, 0.0),
            specular: glm::vec3(1.0, 1.0, 1.0),
            shininess: 10.0,
            material_type: MaterialType::ReflectAndPhong,
        },
        Material {
            name: String::from("transparent"),
            emissive: glm::vec3(0.0, 0.0, 0.0),
            ambient: glm::vec3(0.0, 0.0, 0.0),
            diffuse: glm::vec3(0.0, 0.0, 0.0),
            specular: glm::vec3(0.2, 0.2, 0.2),
            shininess: 100.0,
            material_type: MaterialType::ReflectAndRefract {
                index_inner: 1.5,
                index_outer: 1.0
            },
        },
    ]
}

fn test_triangles(materials: &[Material]) -> Vec<Triangle> {
    vec![
        Triangle::new([glm::vec3(-5.0, 1.25, -5.0),
                          glm::vec3(5.0, 1.25, -5.0),
                          glm::vec3(0.0, -3.75, -5.0)],
                      [zero(); 3],
                      materials.iter().find(|&material| {
                          material.name == "some_shiny_white"
                      }).unwrap()
        ),
        Triangle::new([glm::vec3(-5.0, -2.5, -5.0),
                          glm::vec3(5.0, -2.5, -5.0),
                          glm::vec3(0.0, 2.5, -5.0)
                      ],
                      [zero(); 3],
                      materials.iter().find(|&material| {
                          material.name == "some_shiny_blue"
                      }).unwrap(),
        ),
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
                material.name == "transparent"
            }).unwrap()
        },
        Sphere {
            center: glm::vec3(0.0, 2.5, -5.0),
            radius: 1.0,
            material: materials.iter().find(|&material| {
                material.name == "reflective"
            }).unwrap()
        },
        Sphere {
            center: glm::vec3(0.0, 0.0, 0.0),
            radius: 1.0,
            material: materials.iter().find(|&material| {
                material.name == "some_shiny_yellow"
            }).unwrap()
        }
    ]
}

fn test_instanced_spheres<'a>(materials: &'a[Material], spheres: &'a[Sphere]) -> Vec<Instance<'a, 'a, Sphere<'a>>> {
    let offset = glm::vec3(-2.0, -1.0, -5.0);
    let orientation = glm::vec3(0.0, 0.0, 0.0);
    let scale = glm::vec3(1.0, 1.0, 1.0);

    use raytracing::transform::*;

    let rotation_scale_transform = matrix::scaling(&scale) * matrix::rotation(orientation.y, orientation.x, orientation.z);
    let rotation_scale_transform_inverse = glm::inverse(&rotation_scale_transform);

    let model_transform = matrix::model(&offset, &orientation, &scale);
    let model_transform_inverse = glm::inverse(&model_transform);

    let material_override = materials.iter().find(|&material| {
        material.name == "some_shiny_red"
    });

    vec![
        Instance {
            primitive: &spheres[4],
            rotation_scale: rotation_scale_transform,
            rotation_scale_inverse: rotation_scale_transform_inverse,
            model: model_transform,
            model_inverse: model_transform_inverse,
            material_override
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