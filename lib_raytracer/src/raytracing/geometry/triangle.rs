use crate::raytracing::Material;
use nalgebra_glm as glm;

pub struct Triangle<'a> {
    pub vertices: [glm::Vec3; 3],
    pub normals: [glm::Vec3; 3],
    normal: glm::Vec3,

    pub material: &'a Material,
}

impl Triangle<'_> {
    pub fn new(vertices: [glm::Vec3; 3], normals: [glm::Vec3; 3], material: &Material) -> Triangle {
        Triangle {
            vertices,
            normals,
            normal: Triangle::calculate_unit_normal(&vertices),
            material: material
        }
    }

    pub fn normal(&self) -> &glm::Vec3 {
        &self.normal
    }

    // as per https://en.wikipedia.org/wiki/Right-hand_rule
        fn calculate_unit_normal(vertices: &[glm::Vec3; 3]) -> glm::Vec3 {
        let (a, b, c) = (&vertices[0], &vertices[1], &vertices[2]);
        let ac = *c - *a;
        let ab = *b - *a;
        glm::normalize(&glm::cross(&ac, &ab))
    }
}