use tinyvec::*;
use crate::raytracing::AABB;

pub type NodeIndex = usize;

pub enum NodeType {
    Node {
        child_left: NodeIndex,
        child_right: NodeIndex
    },
    Leaf {
        triangle_indices: ArrayVec<[usize; Node::LEAF_TRIANGLES]>,
    },
}

pub struct Node {
    pub aabb: AABB,
    pub content: NodeType
}

impl Node {
    pub const LEAF_TRIANGLES: usize = 5;

    pub fn create_node(aabb: AABB,
                       child_left: NodeIndex,
                       child_right: NodeIndex) -> Self {
        Node {
            aabb,
            content: NodeType::Node { child_left, child_right },
        }
    }

    pub fn create_leaf(aabb: AABB,
                       triangle_indices: impl Clone + Iterator<Item=usize>) -> Self {
        let triangles_len = triangle_indices.clone().count();
        if triangles_len > Node::LEAF_TRIANGLES {
            panic!("Tried to create BVH leaf node with {triangles_len} triangles, higher than the maximum of {}", Node::LEAF_TRIANGLES)
        }
        let triangle_indices = ArrayVec::from_iter(triangle_indices);
        Node {
            aabb,
            content: NodeType::Leaf { triangle_indices },
        }
    }
}