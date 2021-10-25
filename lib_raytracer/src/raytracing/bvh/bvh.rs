use crate::raytracing::bvh::builder::Builder;
use crate::raytracing::bvh::node::{Node, NodeIndex};
use crate::raytracing::Triangle;

pub struct BVH {
    nodes: Vec<Node>,
    max_depth: usize,
}

impl BVH {
    pub fn from(triangles: Vec<Triangle>) -> BVH {
        let (nodes, max_depth) = Builder::build_bvh(triangles);
        BVH { nodes, max_depth }
    }

    pub fn get_node(&self, index: NodeIndex) -> &Node {
        &self.nodes[index]
    }

    pub fn get_root(&self) -> &Node {
        self.get_node(0)
    }

    pub fn get_max_depth(&self) -> usize {
        self.max_depth
    }
}