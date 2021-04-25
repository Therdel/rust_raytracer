use crate::raytracing::Material;

pub struct Hitpoint<'material> {
    pub t: f32, // ray distance
    pub position: glm::Vec3,
    pub material: &'material Material
}
