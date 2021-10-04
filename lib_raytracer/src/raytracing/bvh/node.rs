use crate::raytracing::{AABB, Triangle};

enum NodeType {
    Node {
        child_left: Box<Node>,
        child_right: Box<Node>
    },
    Leaf {
        triangles: Vec<Triangle>
    }
}

pub struct Node {
    aabb: AABB,
    content: NodeType
}

impl Node {
    pub fn create_node(aabb: AABB,
                       child_left: Box<Node>, child_right: Box<Node>) -> Node {
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