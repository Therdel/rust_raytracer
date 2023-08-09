use std::io;
use crate::object_file::WindingOrder;
use crate::raytracing::{Mesh, MaterialIndex};

pub trait MeshLoader {
    fn load(&self, name: &str, file_name: &str,
            material: MaterialIndex,
            winding_order: WindingOrder) -> io::Result<Mesh>;
}