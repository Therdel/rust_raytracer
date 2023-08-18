use wasm_bindgen::prelude::*;
use std::io;
use std::io::Cursor;

use lib_raytracer::object_file;
use lib_raytracer::object_file::WindingOrder;
use lib_raytracer::scene_file::MeshLoader;
use lib_raytracer::raytracing::{bvh, MaterialIndex, Mesh, MeshTriangle};

#[wasm_bindgen]
extern "C" {
    /// Rust --> JS glue adapted from [wasm-bindgen guide](https://rustwasm.github.io/wasm-bindgen/examples/import-js.html)
    pub type AssetStore;

    #[wasm_bindgen(method)]
    fn get(this: &AssetStore, key: &str) -> JsValue;
}

impl AssetStore {
    pub fn get_asset_bytes(&self, name: &str) -> Option<Vec<u8>> {
        let js_val = self.get(name);
        Self::asset_js_value_to_bytes(js_val)
    }

    fn asset_js_value_to_bytes(value: JsValue) -> Option<Vec<u8>> {
        if value.is_undefined() || value.is_null() {
            return None;
        }

        let buffer = value.dyn_into::<js_sys::SharedArrayBuffer>().ok()?;
        let array = js_sys::Uint8Array::new(&buffer);
        let mut vec = vec![0u8; array.length() as usize];
        array.copy_to(&mut vec);
        Some(vec)
    }
}

impl MeshLoader for &AssetStore {
    fn load(&self, name: &str, file_name: &str, material: MaterialIndex,
            winding_order: WindingOrder,
            mesh_triangles: &mut Vec<MeshTriangle>,
            mesh_bvh_nodes: &mut Vec<bvh::Node>) -> io::Result<Mesh> {
        let mesh_obj = self.get_asset_bytes(file_name);
        let Some(mesh_obj) = mesh_obj else {
            panic!("Loading mesh '{file_name}' was undefined")
        };
        let mut mesh_obj_bufread = Cursor::new(mesh_obj);

        object_file::load_mesh(name.to_string(),
                               &mut mesh_obj_bufread,
                               material,
                               winding_order,
                               mesh_triangles,
                               mesh_bvh_nodes)
    }
}