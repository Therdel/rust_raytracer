use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

use lib_raytracer::object_file;
use lib_raytracer::object_file::WindingOrder;
use lib_raytracer::scene_file::MeshLoader;
use lib_raytracer::raytracing::{bvh, MaterialIndex, Mesh, MeshTriangle};

pub struct FilesystemMeshLoader {
    pub model_dir: PathBuf
}

impl MeshLoader for FilesystemMeshLoader {
    fn load(&self, name: &str, file_name: &str,
            material: MaterialIndex,
            winding_order: WindingOrder,
            mesh_triangles: &mut Vec<MeshTriangle>,
            mesh_bvh_nodes: &mut Vec<bvh::Node>) -> io::Result<Mesh> {
        let path = self.model_dir.join(file_name);
        let mut obj_file = BufReader::new(File::open(path)?);

        object_file::load_mesh(name.to_string(),
                               &mut obj_file,
                               material,
                               winding_order,
                               mesh_triangles,
                               mesh_bvh_nodes)
    }
}