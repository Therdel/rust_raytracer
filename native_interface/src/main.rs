mod write_png;

use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use lib_raytracer::exercise1::{Canvas, Scene, object_file};
use lib_raytracer::raytracing::{Triangle, Plane, Sphere, Light, Camera, LightColor, Material, MaterialType, color::*, Instance, raytracer::{Raytracer, Public}, Mesh};
use nalgebra_glm as glm;
use rayon::prelude::*;
use std::time::Instant;
use num_traits::zero;
use lib_raytracer::utils::AliasArc;
use write_png::*;

const IMAGE_PATH: &'static str = "render.png";

fn main() {
    let materials = test_materials();
    let planes = test_planes(&materials);
    let spheres = test_spheres(&materials);
    let triangles = test_triangles(&materials);
    let meshes = test_meshes(&materials);
    let mesh_instances = test_instanced_meshes(&materials, &meshes);

    let mut scene = Scene {
        camera: test_camera(3640, 2160),
        background: Color::urple(),
        lights: test_lights(),
        planes,
        spheres,
        triangles,
        meshes,
        mesh_instances,
        materials,
    };

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

fn test_lights() -> AliasArc<Vec<Light>, [Light]> {
    let arc = Arc::new(vec![
        Light {
            position: glm::vec4(1.0, 5.0, 1.0, 1.0), // directional
            color: LightColor {
                ambient: glm::vec3(0.1, 0.1, 0.1),
                diffuse: glm::vec3(0.5, 0.5, 0.5),
                specular: glm::vec3(0.5, 0.5, 0.5),
            }
        }
    ]);
    AliasArc::new(arc, |vec|vec.as_slice())
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

fn test_materials() -> AliasArc<Vec<Material>, [Material]> {
    let arc = Arc::new(vec![
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
    ]);
    AliasArc::new(arc, |vec|vec.as_slice())
}

fn get_material(materials: AliasArc<Vec<Material>, [Material]>, name: &str) -> Option<AliasArc<Vec<Material>, Material>> {
    let index = materials
        .iter()
        .enumerate()
        .find(|&(index, material)| {
            material.name == name
        })
        .map(|(index, _)|index)?;

    let materials_arc = AliasArc::into_parent(materials);
    let alias_arc = AliasArc::new(materials_arc, |vec|&vec[index]);
    Some(alias_arc)
}

fn get_mesh(meshes: AliasArc<Vec<Mesh>, [Mesh]>, name: &str) -> Option<AliasArc<Vec<Mesh>, Mesh>> {
    let index = meshes
        .iter()
        .enumerate()
        .find(|&(index, mesh)| {
            mesh.id == name
        })
        .map(|(index, _)|index)?;

    let mesh_arc = AliasArc::into_parent(meshes);
    let alias_arc = AliasArc::new(mesh_arc, |vec|&vec[index]);
    Some(alias_arc)
}

fn test_triangles(materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Triangle>, [Triangle]> {
    let arc = Arc::new(vec![
        Triangle::new([glm::vec3(-5.0, 1.25, -5.0),
                          glm::vec3(5.0, 1.25, -5.0),
                          glm::vec3(0.0, -3.75, -5.0)],
                      [zero(); 3],
                      get_material(materials.clone(), "some_shiny_white").unwrap()
        ),
        Triangle::new([glm::vec3(-5.0, -2.5, -5.0),
                          glm::vec3(5.0, -2.5, -5.0),
                          glm::vec3(0.0, 2.5, -5.0)
                      ],
                      [zero(); 3],
                      get_material(materials.clone(), "some_shiny_blue").unwrap(),
        ),
    ]);
    AliasArc::new(arc, Vec::as_slice)
}

fn test_planes(materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Plane>, [Plane]> {
    let arc = Arc::new(vec![
        Plane {
            normal: glm::vec3(0.0, -1.0, 0.0),
            distance: 5.0,
            material: get_material(materials.clone(), "some_shiny_green").unwrap(),
        }
    ]);
    AliasArc::new(arc, Vec::as_slice)
}

fn test_spheres(materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Sphere>, [Sphere]> {
    let arc = Arc::new(vec![
        Sphere {
            center: glm::vec3(0.0, 1.0, -5.0),
            radius: 0.5,
            material: get_material(materials.clone(), "some_shiny_red").unwrap(),
        },
        Sphere {
            center: glm::vec3(0.0, 0.0, -4.0),
            radius: 0.5,
            material: get_material(materials.clone(), "some_shiny_red").unwrap(),
        },
        Sphere {
            center: glm::vec3(0.0, -1.0, -3.0),
            radius: 0.5,
            material: get_material(materials.clone(), "transparent").unwrap(),
        },
        Sphere {
            center: glm::vec3(0.0, 2.5, -5.0),
            radius: 1.0,
            material: get_material(materials.clone(), "reflective").unwrap(),
        }
    ]);
    AliasArc::new(arc, Vec::as_slice)
}

fn test_meshes(materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Mesh>, [Mesh]> {
    use lib_raytracer::exercise1::object_file::WindingOrder;


    let material = get_material(materials.clone(), "some_shiny_white").unwrap();
    let path = PathBuf::from("res/models/sphere_low.obj");
    let mut obj_file = BufReader::new(File::open(path).unwrap());
    let mesh = object_file::load_mesh("sphere_low".to_string(),
                                      &mut obj_file,
                                      material, WindingOrder::CounterClockwise);
    let arc = Arc::new(vec![
        mesh.unwrap()
    ]);
    AliasArc::new(arc, Vec::as_slice)
}

fn test_instanced_meshes(materials: &AliasArc<Vec<Material>, [Material]>,
                         meshes: &AliasArc<Vec<Mesh>, [Mesh]>) -> AliasArc<Vec<Instance<Mesh>>, [Instance<Mesh>]> {
    let material_override = get_material(materials.clone(), "reflective");
    let mesh = get_mesh(meshes.clone(), "sphere_low").unwrap();

    let offset = glm::vec3(-1.0, -1.0, -2.0);
    let orientation = glm::vec3(0.0, 0.0, 0.0);
    let scale = glm::vec3(1.0, 1.0, 1.0);

    let arc = Arc::new(vec![
        Instance::new(mesh, offset, orientation, scale, material_override)
    ]);
    AliasArc::new(arc, Vec::as_slice)
}