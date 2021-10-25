use crate::raytracing::{AABB, Triangle};

pub type NodeIndex = usize;

pub type OptNodeIndex = Option<NodeIndex>;

pub enum NodeType {
    Node {
        child_left: OptNodeIndex,
        child_right: OptNodeIndex
    },
    Leaf {
        triangles: Vec<Triangle>
    },
}

pub struct Node {
    pub aabb: AABB,
    pub content: NodeType
}

impl Node {
    pub fn create_node(aabb: AABB,
                       child_left: OptNodeIndex, child_right: OptNodeIndex) -> Node {
        Node {
            aabb,
            content: NodeType::Node { child_left, child_right },
        }
    }

    pub fn create_leaf(aabb: AABB,
                       triangles: Vec<Triangle>) -> Node {
        Node {
            aabb,
            content: NodeType::Leaf { triangles },
        }
    }
}