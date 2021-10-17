use crate::raytracing::Material;
use nalgebra_glm as glm;
use crate::utils::AliasRc;

pub struct Plane {
    pub normal: glm::Vec3,
    pub distance: f32,

    pub material: AliasRc<Vec<Material>, Material>,
}