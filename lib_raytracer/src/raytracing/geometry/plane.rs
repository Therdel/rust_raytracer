use std::marker::PhantomData;

use crate::raytracing::MaterialIndex;
use nalgebra_glm as glm;

pub struct Plane {
    pub normal: glm::Vec3,
    pub distance: f32,

    pub material: MaterialIndex,
    _force_constructor_use: PhantomData<()>
}

impl Plane {
    pub fn new(normal: glm::Vec3, distance: f32,
               material: MaterialIndex) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
            material,
            _force_constructor_use: PhantomData::default()
        }
    }
}