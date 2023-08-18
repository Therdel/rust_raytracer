use std::ops::Range;

use nalgebra_glm as glm;
use num_traits::Float;

use crate::raytracing::{AABB, MeshTriangle, Triangle};
use crate::raytracing::bvh::{BVH, hull::Hull, node::Node};
use crate::raytracing::bvh::node::{NodeIndex, NodeType};

type TriangleIndices = Vec<usize>;

enum NodeChild {
    Right,
    Left,
}

struct BuildJob {
    parent_index: usize,
    child_to_update: NodeChild,
    triangle_indices: TriangleIndices,
    current_depth: usize,
}

pub struct Builder<'a> {
    max_depth: usize,
    job_stack: Vec<BuildJob>,

    mesh_triangles: &'a [MeshTriangle],
    nodes: &'a mut Vec<Node>,
}

impl Builder<'_> {
    pub fn build_bvh(mesh_triangle_indices: Range<usize>,
                     mesh_triangles: &[MeshTriangle],
                     mesh_bvh_nodes: &mut Vec<Node>) -> super::BVH {
        let nodes_first_index = mesh_bvh_nodes.len();
        let mut builder = Builder {
            max_depth: 0,
            job_stack: vec![],
            mesh_triangles,
            nodes: mesh_bvh_nodes,
        };
        
        let triangle_indices: TriangleIndices = mesh_triangle_indices.collect();
        let root_index = builder.build_node_queue_children(triangle_indices, 0);
        if root_index != nodes_first_index {
            panic!("BVH root node created in wrong position");
        }

        while let Some(job) = builder.job_stack.pop() {
            let child_index = builder.build_node_queue_children(job.triangle_indices, job.current_depth);
            let parent = builder.get_node_mut(job.parent_index);

            match &mut parent.content {
                NodeType::Node { child_left, child_right } =>
                    match job.child_to_update {
                        NodeChild::Right => *child_right = child_index,
                        NodeChild::Left => *child_left = child_index
                    },
                NodeType::Leaf { .. } => panic!("Tried to attach child to leaf node")
            }
        }

        let max_depth = builder.max_depth;
        let nodes_last_index = mesh_bvh_nodes.len();
        let bvh_node_indices = Range { start: nodes_first_index, end: nodes_last_index };

        BVH { bvh_node_indices, max_depth }
    }

    fn get_node_mut(&mut self, index: NodeIndex) -> &mut Node {
        &mut self.nodes[index]
    }

    fn get_triangle(&self, index: usize) -> &MeshTriangle {
        &self.mesh_triangles[index]
    }

    /// Returns index of given (Mesh)Triangle of ```self.mesh_triangles```
    /// ❗SAFETY❗: Caller must ensure that ```triangle``` is an element of ```self.mesh_triangles```.
    unsafe fn get_mesh_triangle_index(&self, triangle: &Triangle) -> usize {
        let address = triangle as *const _ as usize;
        let triangles_base = self.mesh_triangles.as_ptr() as usize;

        // ❗SAFETY❗: Just calculating pointers, no dereferencing.
        let stride = unsafe {
            let triangle_0_addr = self.mesh_triangles.get_unchecked(0) as *const _ as usize;
            let triangle_1_addr = self.mesh_triangles.get_unchecked(1) as *const _ as usize;
            triangle_1_addr - triangle_0_addr
        };

        (address - triangles_base) / stride
    }

    fn iterate_triangles<'a>(&'a self, triangle_indices: &'a TriangleIndices) -> impl Clone + Iterator<Item=&'a Triangle> {
        triangle_indices.iter()
            .map(|index| &self.get_triangle(*index).0)
    }

    // TODO: Could this work using iterators only - without intermediate Vecs?
    fn split_primitives<'b, 'a:'b>(&self, triangle_indices: TriangleIndices, volume_aabb: &AABB) -> (TriangleIndices, TriangleIndices) {
        let triangles_iter = self.iterate_triangles(&triangle_indices);
        let centroid_range = triangles_iter.clone().map(Triangle::centroid);

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
        let bin_width = axis_extent / BVH::BINS_PER_LAYER as f32;
        let get_bin_id = |triangle: &Triangle| {
            let pos_on_axis_absolute = get_axis(&triangle.centroid());
            let pos_on_axis_relative = pos_on_axis_absolute - axis_begin;
            pos_on_axis_relative / bin_width
        };

        // iterator of triangles left of a bin, exclusive
        let left_iter = |bin_id: usize|
            triangles_iter.clone().filter(move |triangle| get_bin_id(triangle) < bin_id as f32);
        // iterator of triangles right of a bin, inclusive
        let right_iter = |bin_id: usize|
            triangles_iter.clone().filter(move |triangle| get_bin_id(triangle) >= bin_id as f32);

        let mut min_split_costs = f32::infinity();
        let mut min_split_costs_bin = None;
        for split_bin in 1..BVH::BINS_PER_LAYER {
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
            // ❗SAFETY❗: All triangles came from iterating ```self.mesh_triangles```,
            //  thus it's ```self.get_mesh_triangle_index```
            unsafe {
                let left_triangle_indices = left_iter(min_split_costs_bin)
                    .map(|triangle| self.get_mesh_triangle_index(triangle))
                    .collect();
                let right_triangle_indices = right_iter(min_split_costs_bin)
                    .map(|triangle| self.get_mesh_triangle_index(triangle))
                    .collect();
    
                (left_triangle_indices, right_triangle_indices)
            }
        } else {
            panic!("BVH creation failed: All split costs were infinite")
        }
    }

    fn build_node_queue_children(&mut self, triangle_indices: TriangleIndices, current_depth: usize) -> NodeIndex {
        if current_depth > self.max_depth {
            self.max_depth = current_depth;
        }
        let triangles = triangle_indices.iter()
            .map(|index| &self.mesh_triangles[*index].0);
        let aabb = AABB::from_triangles(triangles)
            .expect("Tried to build leaf node with 0 primitives");

        let node;
        let node_index = self.nodes.len();
        if triangle_indices.len() <= Node::LEAF_TRIANGLES {
            node = Node::create_leaf(aabb, triangle_indices.into_iter());
        } else {
            let (left, right) = self.split_primitives(triangle_indices, &aabb);
            if left.is_empty() || right.is_empty() {
                // shouldn't happen:
                // if a child is empty, then the other contains all triangles of the parent
                // the same triangles lead to the same split, leading to infinite recursion
                panic!("BVH builder: Aborting infinite recursion - left or right child of node is empty")
            }

            node = Node::create_node(aabb, NodeIndex::MAX, NodeIndex::MAX);

            self.job_stack.push(BuildJob {
                parent_index: node_index,
                child_to_update: NodeChild::Left,
                triangle_indices: left,
                current_depth: current_depth + 1,
            });
            self.job_stack.push(BuildJob {
                parent_index: node_index,
                child_to_update: NodeChild::Right,
                triangle_indices: right,
                current_depth: current_depth + 1,
            });
        }

        self.nodes.push(node);
        node_index
    }
}