use std::marker::PhantomData;

use crate::raytracing::Material;
use nalgebra_glm as glm;
use crate::utils::AliasArc;

pub struct Plane {
    pub normal: glm::Vec3,
    pub distance: f32,

    pub material: AliasArc<Vec<Material>, Material>,
    _force_constructor_use: PhantomData<()>
}

impl Plane {
    pub fn new(normal: glm::Vec3, distance: f32,
               material: AliasArc<Vec<Material>, Material>) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
            material,
            _force_constructor_use: PhantomData::default()
        }
    }
}