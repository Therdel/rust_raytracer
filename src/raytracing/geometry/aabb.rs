pub struct AABB {
    pub min: glm::Vec3,
    pub max: glm::Vec3
}

impl AABB {
    pub fn surface_area(&self) -> f32 {
        let extent = self.max - self.min;

        // cuboid sides
        let xy = extent.x * extent.y;
        let yz = extent.y * extent.z;
        let xz = extent.x * extent.z;

        // every side exists twice
        2.0 * xy + 2.0 * yz + 2.0 * xz
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::zero;

    #[test]
    fn unit_cube() {
        let cube = AABB { min: zero(), max: glm::vec3(1.0, 1.0, 1.0) };
        assert_eq!(cube.surface_area(), 6.0);
    }

    #[test]
    fn cube_sidelen_two() {
        let cube = AABB { min: zero(), max: glm::vec3(2.0, 2.0, 2.0) };
        assert_eq!(cube.surface_area(), 4.0*6.0);
    }
}