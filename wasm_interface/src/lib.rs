use std::os::raw::c_void;
use std::mem;
use lib_raytracer::exercise1::{Scene};
use lib_raytracer::raytracing::{Triangle, Plane, Sphere, Light, Camera, LightColor, Material, MaterialType, color::*, Instance, raytracer::{Raytracer, Public}, Mesh};
use num_traits::zero;

// In order to work with the memory we expose (de)allocation methods
#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut c_void {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    mem::forget(buf);
    return ptr as *mut c_void;
}

#[no_mangle]
pub extern "C" fn dealloc(ptr: *mut c_void, size: usize) {
    unsafe  {
        let _buf = Vec::from_raw_parts(ptr, 0, size);
    }
}

use nalgebra_glm as glm;
use std::slice;

pub type ColorRgba = glm::Vec4;
pub type ColorRgbaU8 = [u8; 4]; // TODO: replace with glm::U8Vec4?

pub trait QuantizeToU8 {
    fn quantize(&self) -> ColorRgbaU8;
}

impl QuantizeToU8 for ColorRgba {
    fn quantize(&self) -> ColorRgbaU8 {
        let mut clamped_color = glm::clamp(self, 0.0, 1.0);
        clamped_color = clamped_color * 255.;

        [
            clamped_color.x as u8,
            clamped_color.y as u8,
            clamped_color.z as u8,
            clamped_color.w as u8
        ]
    }
}


#[no_mangle]
pub extern "C" fn render(ptr: *mut u8, width: usize, height: usize) {
    let ptr_color = ptr as *mut ColorRgbaU8;
    let slice = unsafe { slice::from_raw_parts_mut(ptr_color, width*height  ) };

    let background = Color::urple();
    let mut scene = Scene {
        camera: test_camera(width, height),
        background,
        lights: test_lights(),
        planes: vec![],
        spheres: vec![],
        triangles: vec![],
        meshes: vec![],
        mesh_instances: vec![],

        materials: vec![],

    };
    scene.materials = test_materials();
    scene.planes = test_planes(&scene.materials);
    scene.spheres = test_spheres(&scene.materials);
    scene.triangles = test_triangles(&scene.materials);

    let raytracer = Raytracer::new(&scene);
    for y in 0..height {
        for x in 0..width {
            let coordinate = glm::vec2(x as _, y as _);
            let ray = raytracer.generate_primary_ray(&coordinate);

            let color = match raytracer.raytrace(&ray) {
                Some(hit_color) => hit_color,
                None => background
            };
            let color = glm::vec4(color.x, color.y, color.z, 1.0);

            let max_y_index = height - 1;
            let y_inverted = max_y_index - y;
            let offset = x + width * y_inverted;

            slice[offset] = color.quantize();
        }
    }
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
        }
    ]
}