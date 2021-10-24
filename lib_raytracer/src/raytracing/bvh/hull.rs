use nalgebra_glm as glm;

use crate::raytracing::{AABB, Triangle};

pub trait Hull {
    type HullGeometry;

    // TODO: Remove duplication
    fn from_vertices(vertices: impl Iterator<Item=glm::Vec3>) -> Option<Self::HullGeometry>;
    fn from_vertices_refs<'a>(vertices: impl Iterator<Item=&'a glm::Vec3>) -> Option<Self::HullGeometry>;
    fn from_triangles<'a>(triangles: impl Iterator<Item=&'a Triangle>) -> Option<Self::HullGeometry>;
}

impl Hull for AABB {
    type HullGeometry = AABB;

    fn from_vertices(mut vertices: impl Iterator<Item=glm::Vec3>) -> Option<Self::HullGeometry> {
        let first_point = vertices.next()?;
        let mut result = AABB {
            min: first_point,
            max: first_point
        };

        for vertex in vertices {
            result.min.x = f32::min(result.min.x, vertex.x);
            result.min.y = f32::min(result.min.y, vertex.y);
            result.min.z = f32::min(result.min.z, vertex.z);

            result.max.x = f32::max(result.max.x, vertex.x);
            result.max.y = f32::max(result.max.y, vertex.y);
            result.max.z = f32::max(result.max.z, vertex.z);
        }

        Some(result)
    }

    fn from_vertices_refs<'a>(mut vertices: impl Iterator<Item=&'a glm::Vec3>) -> Option<Self::HullGeometry> {
        let first_point = vertices.next()?;
        let mut result = AABB {
            min: *first_point,
            max: *first_point
        };

        for vertex in vertices {
            result.min.x = f32::min(result.min.x, vertex.x);
            result.min.y = f32::min(result.min.y, vertex.y);
            result.min.z = f32::min(result.min.z, vertex.z);

            result.max.x = f32::max(result.max.x, vertex.x);
            result.max.y = f32::max(result.max.y, vertex.y);
            result.max.z = f32::max(result.max.z, vertex.z);
        }

        Some(result)
    }

    fn from_triangles<'a>(triangles: impl Iterator<Item=&'a Triangle>) -> Option<Self::HullGeometry> {
        let vertices = triangles
            .flat_map(|triangle| triangle.vertices.iter());
        Self::from_vertices_refs(vertices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod vertices {
        use super::*;

        #[test]
        fn empty() {
            let vertices: [glm::Vec3; 0] = [];

            let iter = vertices.iter();
            // TODO: why doesn't this work with ```let hull: AABB = Hull::hull(iter);```
            let hull = AABB::from_vertices_refs(iter);
            assert!(hull.is_none());
        }

        #[test]
        fn order_does_not_matter() {
            let zero_vec = glm::vec3(0., 0., 0.);
            let one_vec = glm::vec3(1., 1., 1.);

            let vertices = [zero_vec, one_vec];
            {
                let iter = vertices.iter();
                let hull = AABB::from_vertices_refs(iter);
                assert!(hull.is_some());
                let hull = hull.unwrap();

                assert_eq!(hull.min, zero_vec);
                assert_eq!(hull.max, one_vec);
            }
            {
                let iter_rev = vertices.iter().rev();
                let hull = AABB::from_vertices_refs(iter_rev);
                assert!(hull.is_some());
                let hull = hull.unwrap();

                assert_eq!(hull.min, zero_vec);
                assert_eq!(hull.max, one_vec);
            }
        }
    }
    mod triangles {
        use std::sync::Arc;
        use num_traits::zero;

        use crate::raytracing::{AABB, Material, MaterialType, Triangle};
        use crate::utils::AliasArc;

        use super::*;

        fn test_material() -> AliasArc<Vec<Material>, Material> {
            let arc = Arc::new(vec![Material {
                name: String::from("marriage_material"),
                emissive: zero(),
                ambient: zero(),
                diffuse: zero(),
                specular: zero(),
                shininess: 0.0,
                material_type: MaterialType::Phong
            }]);
            AliasArc::new(arc, |vec|vec.first().unwrap())
        }

        #[test]
        fn smoke_test() {
            let material = test_material();
            let vertices = [
                glm::vec3(-1.0, 2.0, 3.0),
                glm::vec3(1.0, -2.0, 0.0),
                glm::vec3(0.0, 0.0, 0.0)
            ];
            let triangles = [Triangle::new(vertices, [zero(); 3], material)];
            let hull = AABB::from_triangles(triangles.iter());
            assert!(hull.is_some());
            let hull = hull.unwrap();

            assert_eq!(hull.min, glm::vec3(-1.0, -2.0, 0.0));
            assert_eq!(hull.max, glm::vec3(1.0, 2.0, 3.0));
        }
    }
}

