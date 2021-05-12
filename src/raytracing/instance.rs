use crate::raytracing::{Material, Intersect};

pub struct Instance<'primitive, 'material, Primitive: Intersect> {
    pub primitive: &'primitive Primitive,
    pub rotation_scale: glm::Mat4,
    pub rotation_scale_inverse: glm::Mat4,
    pub model: glm::Mat4,
    pub model_inverse: glm::Mat4,
    pub material_override: Option<&'material Material>
}