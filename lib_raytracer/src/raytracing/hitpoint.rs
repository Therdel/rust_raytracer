use crate::raytracing::MaterialIndex;
use nalgebra_glm as glm;

pub struct Hitpoint {
    pub t: f32, // ray distance
    pub position: glm::Vec3,
    pub hit_normal: glm::Vec3,
    pub position_for_refraction: glm::Vec3,
    pub on_frontside: bool,

    pub material: MaterialIndex,
}
