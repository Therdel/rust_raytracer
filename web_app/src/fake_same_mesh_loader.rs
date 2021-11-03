use std::io;
use std::io::Cursor;

use lib_raytracer::exercise1::object_file;
use lib_raytracer::exercise1::object_file::WindingOrder;
use lib_raytracer::exercise1::scene_file::MeshLoader;
use lib_raytracer::raytracing::{Material, Mesh};
use lib_raytracer::utils::AliasArc;

pub struct FakeSameMeshLoader<'a> {
    pub mesh_obj: &'a [u8]
}

impl MeshLoader for FakeSameMeshLoader<'_> {
    fn load(&self, name: &str, _file_name: &str, material: AliasArc<Vec<Material>, Material>,
            winding_order: WindingOrder) -> io::Result<Mesh> {
        let mut mesh_obj_bufread = Cursor::new(self.mesh_obj);

        object_file::load_mesh(name.to_string(),
                               &mut mesh_obj_bufread,
                               material,
                               winding_order)
    }
}