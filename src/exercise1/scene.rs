use crate::raytracing::{self, Ray, Hitpoint};

pub struct Scene {
}

impl raytracing::Intersect for Scene {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        todo!()
    }
}