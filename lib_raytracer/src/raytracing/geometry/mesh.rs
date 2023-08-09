use crate::raytracing::bvh::BVH;
use crate::raytracing::Triangle;

#[derive(Clone, Copy)]
pub struct MeshIndex(pub usize);

pub struct Mesh {
    pub name: String,
    pub triangles: Vec<Triangle>,
    pub bvh: BVH,
}