use std::io::Cursor;
use std::os::raw::c_void;
use std::mem;
use std::sync::Arc;
use lib_raytracer::exercise1::{Scene, object_file};
use lib_raytracer::raytracing::{Triangle, Plane, Sphere, Light, Camera, LightColor, Material, MaterialType, color::*, Instance, raytracer::{Raytracer, Public}, Mesh, Screen};
use lib_raytracer::utils::AliasArc;
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

fn empty_alias_vec<T>() -> AliasArc<Vec<T>, [T]> {
    AliasArc::new(Default::default(), Vec::as_slice)
}

#[no_mangle]
pub extern "C" fn render(ptr: *mut u8, width: usize, height: usize, mesh_obj_buf: *const u8, mesh_obj_buf_len: usize) {
    let ptr_color = ptr as *mut ColorRgbaU8;
    let slice = unsafe { slice::from_raw_parts_mut(ptr_color, width*height  ) };

    let mesh_obj = unsafe {
        slice::from_raw_parts(mesh_obj_buf, mesh_obj_buf_len)
    };

    let materials = test_materials();
    let planes = test_planes(&materials);
    let spheres = test_spheres(&materials);
    let triangles = test_triangles(&materials);
    let meshes = test_meshes(&materials, mesh_obj);
    let mesh_instances = empty_alias_vec();//test_instanced_meshes(&materials, &meshes);

    let scene = Scene {
        camera: test_camera(),
        screen: Screen {
            pixel_width: width,
            pixel_height: height,
            background: Color::urple()
        },
        lights: test_lights(),
        planes,
        spheres,
        triangles,
        meshes,
        mesh_instances,
        materials,
    };

    let raytracer = Raytracer::new(&scene);
    for y in 0..height {
        for x in 0..width {
            let coordinate = glm::vec2(x as _, y as _);
            let ray = raytracer.generate_primary_ray(&coordinate);

            let color = match raytracer.raytrace(&ray) {
                Some(hit_color) => hit_color,
                None => scene.screen.background
            };
            let color = glm::vec4(color.x, color.y, color.z, 1.0);

            let max_y_index = height - 1;
            let y_inverted = max_y_index - y;
            let offset = x + width * y_inverted;

            slice[offset] = color.quantize();
        }
    }
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
    AliasArc::new(arc, Vec::as_slice)
}

fn test_camera() -> Camera {
    Camera {
        position: glm::vec3(3.0, 0.0, 1.0),
        orientation: glm::vec3(0.0f32.to_radians(),
                               25.0f32.to_radians(),
                               0.0f32.to_radians()),
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
    AliasArc::new(arc, Vec::as_slice)
}

fn get_material(materials: AliasArc<Vec<Material>, [Material]>, name: &str) -> Option<AliasArc<Vec<Material>, Material>> {
    let index = materials
        .iter()
        .enumerate()
        .find(|&(_, material)| {
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
        .find(|&(_, mesh)| {
            mesh.name == name
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

fn test_meshes(materials: &AliasArc<Vec<Material>, [Material]>, mesh_obj: &[u8]) -> AliasArc<Vec<Mesh>, [Mesh]> {
    use lib_raytracer::exercise1::object_file::WindingOrder;


    let material = get_material(materials.clone(), "some_shiny_white").unwrap();
    let mut mesh_obj_bufread = Cursor::new(mesh_obj);
    let mesh = object_file::load_mesh("sphere_low".to_string(),
                                      &mut mesh_obj_bufread,
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