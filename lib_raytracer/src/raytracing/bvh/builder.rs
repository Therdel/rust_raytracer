use nalgebra_glm as glm;
use num_traits::{Float, Zero};

use crate::raytracing::{AABB, Triangle};
use crate::raytracing::bvh::{hull::Hull, node::Node};
use crate::raytracing::bvh::node::{NodeIndex, NodeType};

const BINS_PER_LAYER: usize = 5;
const MAX_PRIMITIVES_PER_LEAF: usize = 5;
type Triangles = Vec<Triangle>;

enum NodeChild {
    Right,
    Left,
}

struct BuildJob {
    parent_index: usize,
    child_to_update: NodeChild,
    triangles: Triangles,
    current_depth: usize,
}

pub struct Builder {
    max_depth: usize,

    job_stack: Vec<BuildJob>,
    nodes: Vec<Node>,
}

impl Builder {
    pub fn build_bvh(triangles: Triangles) -> (Vec<Node>, usize) {
        let mut builder = Builder {
            max_depth: 0,
            job_stack: vec![],
            nodes: vec![]
        };

        let _root_index = builder.build_node_queue_children(triangles, 0);
        while let Some(job) = builder.job_stack.pop() {
            let child_index = builder.build_node_queue_children(job.triangles, job.current_depth);
            let parent = builder.get_node_mut(job.parent_index);

            match &mut parent.content {
                NodeType::Node { child_left, child_right } =>
                    match job.child_to_update {
                        NodeChild::Right => *child_right = Some(child_index),
                        NodeChild::Left => *child_left = Some(child_index)
                    },
                NodeType::Leaf { .. } => panic!("Tried to attach child to leaf node")
            }
        }

        (builder.nodes, builder.max_depth)
    }

    fn get_node_mut(&mut self, index: NodeIndex) -> &mut Node {
        &mut self.nodes[index]
    }

    // TODO: Could this work using iterators only - without intermediate Vecs?
    fn split_primitives<'b, 'a:'b>(triangles: &'b[Triangle], volume_aabb: &AABB) -> (Triangles, Triangles) {
        let centroid_range = triangles.iter().map(Triangle::centroid);
        let centroid_aabb= AABB::from_vertices(centroid_range).expect("Empty triangles iterator");

        let extent = centroid_aabb.max - centroid_aabb.min;

        enum Axis { X, Y, Z }
        let axis =
            if extent.max() == extent.x {
                Axis::X
            } else if extent.max() == extent.y {
                Axis::Y
            } else {
                Axis::Z
            };

        let get_axis = |pos: &glm::Vec3| match axis {
            Axis::X => pos.x,
            Axis::Y => pos.y,
            Axis::Z => pos.z
        };
        let axis_begin = get_axis(&centroid_aabb.min);
        let axis_extent = get_axis(&extent);
        let bin_width = axis_extent / BINS_PER_LAYER as f32;
        let get_bin_id = |triangle: &Triangle| {
            let pos_on_axis_absolute = get_axis(&triangle.centroid());
            let pos_on_axis_relative = pos_on_axis_absolute - axis_begin;
            pos_on_axis_relative / bin_width
        };

        // iterator of triangles left of a bin, exclusive
        let left_iter = |bin_id: usize|
            triangles.iter().filter(move |triangle| get_bin_id(triangle) < bin_id as f32);
        // iterator of triangles right of a bin, inclusive
        let right_iter = |bin_id: usize|
            triangles.iter().filter(move |triangle| get_bin_id(triangle) >= bin_id as f32);

        let mut min_split_costs = f32::infinity();
        let mut min_split_costs_bin = None;
        for split_bin in 0..BINS_PER_LAYER {
            // TODO: Is the upper bound really necessary? The Right cutoff is inclusive, after all.
            //       Though that would mean the triangle's bin would have to be float-equal or bigger than that number.
            //       Let's try it.
            if split_bin.is_zero() {// || split_bin == BINS_PER_LAYER-1 {
                min_split_costs = f32::infinity();
                continue;
            }

            let p_left = match AABB::from_triangles(left_iter(split_bin)) {
                Some(left_hull) => left_hull.surface_area() / volume_aabb.surface_area(),
                None => 0.0
            };
            let p_right = match AABB::from_triangles(right_iter(split_bin)) {
                Some(right_hull) => right_hull.surface_area() / volume_aabb.surface_area(),
                None => 0.0
            };

            let (left_amount, right_amount) = (left_iter(split_bin).count(), right_iter(split_bin).count());

            let cost = p_left * left_amount as f32 + p_right * right_amount as f32;
            if cost < min_split_costs {
                min_split_costs = cost;
                min_split_costs_bin = Some(split_bin);
            }
        }

        if let Some(min_split_costs_bin) = min_split_costs_bin {
            let left_split = left_iter(min_split_costs_bin).cloned().collect();
            let right_split = right_iter(min_split_costs_bin).cloned().collect();

            (left_split, right_split)
        } else {
            panic!("BVH creation failed: All split costs were infinite")
        }
    }

    fn build_leaf(triangles: Triangles) -> Node {
        let aabb = AABB::from_triangles(triangles.iter())
            .expect("Tried to build leaf node with 0 primitives");
        Node::create_leaf(aabb, triangles)
    }

    fn build_node_queue_children(&mut self, triangles: Triangles, current_depth: usize) -> NodeIndex {
        if current_depth > self.max_depth {
            self.max_depth = current_depth;
        }

        if triangles.len() <= MAX_PRIMITIVES_PER_LEAF {
            let node = Self::build_leaf(triangles);
            self.nodes.push(node);
            let index = self.nodes.len() - 1;
            index
        } else {
            // always exists because it exceeds leaf threshold
            let aabb = AABB::from_triangles(triangles.iter()).unwrap();
            let (left, right) = Self::split_primitives(&triangles, &aabb);

            let node = Node::create_node(aabb, None, None);
            self.nodes.push(node);
            let index = self.nodes.len() - 1;
            
            // TODO: Check if this is ever empty and why
            if !left.is_empty() {
                self.job_stack.push(BuildJob {
                    parent_index: index,
                    child_to_update: NodeChild::Left,
                    triangles: left,
                    current_depth: current_depth + 1,
                });
            }
            if !right.is_empty() {
                self.job_stack.push(BuildJob {
                    parent_index: index,
                    child_to_update: NodeChild::Right,
                    triangles: right,
                    current_depth: current_depth + 1,
                });
            }

            index
        }
    }
}