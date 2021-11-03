use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

use lib_raytracer::exercise1::object_file;
use lib_raytracer::exercise1::object_file::WindingOrder;
use lib_raytracer::exercise1::scene_file::MeshLoader;
use lib_raytracer::raytracing::{Material, Mesh};
use lib_raytracer::utils::AliasArc;

pub struct FilesystemMeshLoader {
    pub model_dir: PathBuf
}

impl MeshLoader for FilesystemMeshLoader {
    fn load(&self, name: &str, file_name: &str, material: AliasArc<Vec<Material>, Material>,
            winding_order: WindingOrder) -> io::Result<Mesh> {
        let path = self.model_dir.join(file_name);
        let mut obj_file = BufReader::new(File::open(&path)?);

        object_file::load_mesh(name.to_string(),
                               &mut obj_file,
                               material,
                               winding_order)
    }
}