use std::io;
use crate::object_file::WindingOrder;
use crate::raytracing::{bvh, Mesh, MaterialIndex, MeshTriangle};

pub trait MeshLoader {
    fn load(&self, name: &str, file_name: &str,
            material: MaterialIndex,
            winding_order: WindingOrder,
            mesh_triangles: &mut Vec<MeshTriangle>,
            mesh_bvh_nodes: &mut Vec<bvh::Node>) -> io::Result<Mesh>;
}