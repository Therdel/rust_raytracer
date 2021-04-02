use crate::raytracing::{Ray, Hitpoint, Sphere, Plane, Triangle};

pub trait Intersect {
    type Result;
    /**
     * Tests a `ray` and an object for intersection and returns whether there is one.
     * Returns information of the hitpoint, if any
     **/
    fn intersect(&self, ray: &Ray) -> Option<Self::Result>;
}

impl Intersect for Sphere {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        todo!()
    }
}

impl Intersect for Plane {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        todo!()
    }
}

impl Intersect for Triangle {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        todo!()
    }
}