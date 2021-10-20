use std::io::Cursor;
use std::mem;
use std::os::raw::c_void;
use std::slice;

use nalgebra_glm as glm;
use lib_raytracer::exercise1::scene_file::Parser;
use lib_raytracer::raytracing::{raytracer::{Public, Raytracer}, color::*, Screen};
use lib_raytracer::utils::AliasArc;

use crate::fake_same_mesh_loader::FakeSameMeshLoader;

mod fake_same_mesh_loader;

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
pub extern "C" fn render(ptr: *mut u8, width: usize, height: usize,
                         scene_buf: *const u8, scene_buf_len: usize,
                         mesh_obj_buf: *const u8, mesh_obj_buf_len: usize) {
    let ptr_color = ptr as *mut ColorRgbaU8;
    let slice = unsafe { slice::from_raw_parts_mut(ptr_color, width * height) };

    let mesh_obj = unsafe {
        slice::from_raw_parts(mesh_obj_buf, mesh_obj_buf_len)
    };
    let scene_json = unsafe {
        slice::from_raw_parts(scene_buf, scene_buf_len)
    };

    let mut scene = Parser {
        file_reader: Cursor::new(scene_json),
        mesh_loader: FakeSameMeshLoader { mesh_obj },
    }.parse_json().unwrap();
    scene.screen = Screen {
        pixel_width: width,
        pixel_height: height,
        background: Color::urple()
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