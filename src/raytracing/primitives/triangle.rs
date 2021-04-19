use crate::raytracing::Material;

pub struct Triangle<'a> {
    pub a: glm::Vec3,
    pub b: glm::Vec3,
    pub c: glm::Vec3,
    normal: glm::Vec3,

    pub material: &'a Material,
}

impl Triangle<'_> {
    pub fn new(a: glm::Vec3, b: glm::Vec3, c: glm::Vec3, material: &Material) -> Triangle {
        Triangle {
            a, b, c,
            normal: Triangle::calculate_unit_normal(a, b, c),
            material: material
        }
    }

    pub fn normal(&self) -> &glm::Vec3 {
        &self.normal
    }

    // as per https://en.wikipedia.org/wiki/Right-hand_rule
    fn calculate_unit_normal(a: glm::Vec3, b: glm::Vec3, c: glm::Vec3) -> glm::Vec3 {
        let ac = c - a;
        let ab = b - a;
        glm::normalize(glm::cross(ac, ab))
    }
}