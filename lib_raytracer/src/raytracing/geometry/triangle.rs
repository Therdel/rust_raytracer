use crate::raytracing::MaterialIndex;
use num_traits::zero;
use nalgebra_glm as glm;

#[derive(Clone)]
pub struct Triangle {
    pub vertices: [glm::Vec3; 3],
    pub normals: [glm::Vec3; 3],
    normal: glm::Vec3,

    pub material: MaterialIndex,
}

impl Triangle {
    pub fn new(vertices: [glm::Vec3; 3], normals: [glm::Vec3; 3],
               material: MaterialIndex) -> Triangle {
        Triangle {
            vertices,
            normals,
            normal: Triangle::calculate_unit_normal(&vertices),
            material
        }
    }

    pub fn normal(&self) -> &glm::Vec3 {
        &self.normal
    }

    pub fn centroid(&self) -> glm::Vec3 {
        // barycentric coordinates 1/3 : 1/3 : 1/3
        let mut sum: glm::Vec3 = zero();
        for vertex in &self.vertices {
            sum = sum + *vertex
        }
        // let sum: glm::Vec3 = self.vertices
        //     .iter()
        //     .fold(zero(), |acc, vertex| acc + *vertex);
        sum * (1.0/3.0)
    }

    // as per https://en.wikipedia.org/wiki/Right-hand_rule
    fn calculate_unit_normal(vertices: &[glm::Vec3; 3]) -> glm::Vec3 {
        let (a, b, c) = (&vertices[0], &vertices[1], &vertices[2]);
        let ac = *c - *a;
        let ab = *b - *a;
        glm::normalize(&glm::cross(&ac, &ab))
    }
}