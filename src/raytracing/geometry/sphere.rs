use crate::raytracing::Material;

pub struct Sphere<'a> {
    pub center: glm::Vec3,
    pub radius: f32,

    pub material: &'a Material,
}

impl Sphere<'_> {
    pub fn normal(&self, surface_point: &glm::Vec3) -> glm::Vec3 {
        let surface_normal = *surface_point - self.center;
        glm::normalize(surface_normal)
    }
}