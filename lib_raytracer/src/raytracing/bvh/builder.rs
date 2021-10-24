use crate::raytracing::{Triangle, AABB};
use crate::raytracing::bvh::{node::Node, hull::Hull};
use num_traits::{Zero, Float};
use nalgebra_glm as glm;

const BINS_PER_LAYER: usize = 5;
const MAX_PRIMITIVES_PER_LEAF: usize = 5;
type Triangles = Vec<Triangle>;

struct NodeJob {
    triangles: Triangles
}

pub struct Builder {
    amount_nodes: usize,
    max_depth: usize,

    node_job_stack: Vec<NodeJob>,
}

impl Builder {
    pub fn build_bvh(triangles: Triangles) -> Node {
        todo!()
    }

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
            let id = pos_on_axis_relative / bin_width;
            id
        };

        // iterator of triangles left of a bin, exclusive
        let left_iter = |bin_id: usize|
            triangles.iter().filter(move |triangle| get_bin_id(triangle) < bin_id as f32);
        // iterator of triangles right of a bin, inclusive
        let right_iter = |bin_id: usize|
            triangles.iter().filter(move |triangle| get_bin_id(triangle) >= bin_id as f32);

        let mut split_costs = [0.0; BINS_PER_LAYER];

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
            (left_iter(min_split_costs_bin).cloned().collect::<Triangles>(),
             right_iter(min_split_costs_bin).cloned().collect::<Triangles>())

            // let (mut left_triangles, mut right_triangles) = (vec![], vec![]);
            // for triangle in left_iter(min_split_costs_bin) {
            //     let copy = triangle.clone();
            //     left_triangles.push(copy);
            // }
            //
            // (left_triangles, right_triangles)
        } else {
            panic!("BVH creation failed: All split costs were infinite")
        }
    }

    fn build_leaf(triangles: Triangles) -> Node {
        todo!()
    }

    fn build_node(triangles: Triangles, current_depth: usize) -> Node {
        todo!()
    }
}