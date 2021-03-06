use crate::raytracing::bvh::BVH;
use crate::raytracing::Triangle;

pub struct Mesh {
    pub name: String,
    pub triangles: Vec<Triangle>,
    pub bvh: BVH,
}