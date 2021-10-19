use std::io;
use crate::exercise1::object_file::WindingOrder;
use crate::raytracing::{Material, Mesh};
use crate::utils::AliasArc;

pub trait MeshLoader {
    fn load(&self, name: &str, file_name: &str,
            material: AliasArc<Vec<Material>, Material>,
            winding_order: WindingOrder) -> io::Result<Mesh>;
}