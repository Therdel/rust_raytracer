use crate::raytracing::Material;
use nalgebra_glm as glm;

pub struct Plane<'a> {
    pub normal: glm::Vec3,
    pub distance: f32,

    pub material: &'a Material,
}