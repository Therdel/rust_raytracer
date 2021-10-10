use crate::raytracing::{Material, Intersect, transform::matrix};
use nalgebra_glm as glm;

pub struct Instance<'primitive, 'material, Primitive: Intersect> {
    pub primitive: &'primitive Primitive,
    pub rotation_scale: glm::Mat4,
    pub rotation_scale_inverse: glm::Mat4,
    pub model: glm::Mat4,
    pub model_inverse: glm::Mat4,
    pub material_override: Option<&'material Material>,
}

impl<'primitive, 'material, Primitive: Intersect> Instance<'_, '_, Primitive> {
    pub fn new(
        primitive: &'primitive Primitive,
        position: glm::Vec3,
        orientation: glm::Vec3,
        scale: glm::Vec3,
        material_override: Option<&'material Material>,
    ) -> Instance<'primitive, 'material, Primitive> {
        let rotation_scale_transform = matrix::scaling(&scale) * matrix::rotation(orientation.y, orientation.x, orientation.z);
        let rotation_scale_transform_inverse = glm::inverse(&rotation_scale_transform);

        let model_transform = matrix::model(&position, &orientation, &scale);
        let model_transform_inverse = glm::inverse(&model_transform);

        Instance {
            primitive: primitive,
            rotation_scale: rotation_scale_transform,
            rotation_scale_inverse: rotation_scale_transform_inverse,
            model: model_transform,
            model_inverse: model_transform_inverse,
            material_override,
        }
    }
}