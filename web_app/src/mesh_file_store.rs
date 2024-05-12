use std::io::Cursor;

use lib_raytracer::object_file::{self, WindingOrder};
use lib_raytracer::raytracing::{MaterialIndex, Mesh};
use lib_raytracer::scene_file::MeshLoader;

pub struct MeshFileStore {
    pub meshes: std::collections::HashMap<String, Vec<u8>>
}

impl MeshLoader for &MeshFileStore {
    fn load(&self, name: &str, file_name: &str,
            material: MaterialIndex,
            winding_order: WindingOrder) -> std::io::Result<Mesh> {
        let Some(file_buffer) = self.meshes.get(file_name) else {
            return Err(std::io::ErrorKind::NotFound.into())
        };
        let mut file_bufread = Cursor::new(file_buffer);

        object_file::load_mesh(name.to_string(),
                               &mut file_bufread,
                               material,
                               winding_order)
    }
}