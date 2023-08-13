use std::io;
use std::io::Cursor;

use lib_raytracer::object_file;
use lib_raytracer::object_file::WindingOrder;
use lib_raytracer::scene_file::MeshLoader;
use lib_raytracer::raytracing::{MaterialIndex, Mesh};

use crate::mesh_store::MeshStore;

pub struct MeshFetcher<'a> {
    pub mesh_store: &'a MeshStore
}

impl MeshLoader for MeshFetcher<'_> {
    fn load(&self, name: &str, file_name: &str, material: MaterialIndex,
            winding_order: WindingOrder) -> io::Result<Mesh> {
        let mesh_obj = self.mesh_store.get_mesh(file_name);
        let mut mesh_obj_bufread = Cursor::new(mesh_obj);

        object_file::load_mesh(name.to_string(),
                               &mut mesh_obj_bufread,
                               material,
                               winding_order)
    }
}