use crate::raytracing::{self, Ray, Hitpoint, Sphere, Plane, Triangle, Light, Camera, Material};

pub struct Scene<'a> {
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub planes: Vec<Plane<'a>>,
    pub spheres: Vec<Sphere<'a>>,
    pub triangles: Vec<Triangle<'a>>,

    pub materials: Vec<Material>
}

// impl<'a> Scene<'a> {
    // pub fn get_material(&'a self, name: &str) -> Option<&'a Material> {
        // TODO: Document learnt: Why didn't this work?
        // self.materials.iter().find(|&material| {
        //     material.name == name
        // })

        // TODO: Document learn: Why didn't this work?
        //       Document: missing ```&'a self``` lifetime.
        // let mut result = None;
        // for i in 0..self.materials.len() {
        //     let ref material = self.materials[i];
        //     if material.name == name {
        //         result = Some(material);
        //     }
        // }
        // result
    // }
// }

impl<'a> raytracing::Intersect for Scene<'a> {
    type Result = Hitpoint<'a>;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let mut closest_hitpoint: Option<Self::Result> = None;

        let mut check_hitpoint = |hitpoint: Option<Self::Result>| {
            if let Some(hitpoint) = hitpoint {
                if let Some(ref mut closest_hitpoint) = closest_hitpoint {
                    if hitpoint.t < closest_hitpoint.t {
                        *closest_hitpoint = hitpoint;
                    }
                } else {
                    closest_hitpoint = Some(hitpoint);
                }
            }
        };

        for plane in &self.planes {
            check_hitpoint(plane.intersect(ray));
        }
        for sphere in &self.spheres {
            check_hitpoint(sphere.intersect(ray));
        }
        for triangle in &self.triangles {
            check_hitpoint(triangle.intersect(ray));
        }

        closest_hitpoint
    }
}