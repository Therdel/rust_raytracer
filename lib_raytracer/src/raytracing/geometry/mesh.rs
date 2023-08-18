use std::ops::Range;

use crate::raytracing::bvh::BVH;

#[derive(Clone, Copy)]
pub struct MeshIndex(pub usize);

pub struct Mesh {
    pub name: String,
    pub triangle_indices: Range<usize>,
    pub bvh: BVH,
}