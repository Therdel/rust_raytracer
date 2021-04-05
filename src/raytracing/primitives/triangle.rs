pub struct Triangle {
    pub a: glm::Vec3,
    pub b: glm::Vec3,
    pub c: glm::Vec3,
    normal: glm::Vec3,
}

impl Triangle {
    pub fn new(a: glm::Vec3, b: glm::Vec3, c: glm::Vec3) -> Triangle {
        Triangle {
            a, b, c,
            normal: Triangle::calculate_unit_normal(a, b, c)
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