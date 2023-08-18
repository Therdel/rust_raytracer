mod hull;
mod node;
pub use node::*;
mod builder;

use std::ops::Range;
use crate::raytracing::bvh::builder::Builder;
// TODO: Find out why this use of `Node` un-does the `pub use node::*` earlier
// use crate::raytracing::bvh::node::{Node, NodeIndex};
use crate::raytracing::MeshTriangle;

pub struct BVH {
    pub bvh_node_indices: Range<NodeIndex>,
    pub max_depth: usize,
}

impl BVH {
    const BINS_PER_LAYER: usize = 5;
    pub fn build(mesh_triangle_indices: Range<usize>,
                 mesh_triangles: &[MeshTriangle],
                 mesh_bvh_nodes: &mut Vec<Node>) -> Self {
        let bvh =
            Builder::build_bvh(mesh_triangle_indices, mesh_triangles, mesh_bvh_nodes);
        println!("BVH with {} nodes and max_depth = {}", bvh.bvh_node_indices.len(), bvh.max_depth);
        bvh
    }
}