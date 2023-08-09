use crate::raytracing::MaterialIndex;
use nalgebra_glm as glm;

pub struct Sphere {
    pub center: glm::Vec3,
    pub radius: f32,

    pub material: MaterialIndex,
}

impl Sphere {
    pub fn normal(&self, surface_point: &glm::Vec3) -> glm::Vec3 {
        let surface_normal = *surface_point - self.center;
        glm::normalize(&surface_normal)
    }
}