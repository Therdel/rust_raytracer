use crate::raytracing::{self, Ray, Hitpoint, Sphere, Plane, Triangle, Light, Camera, Material, Instance, Mesh};
use crate::raytracing::color::ColorRgb;
use crate::utils;

pub struct Scene<'a> {
    pub camera: Camera,
    pub background: ColorRgb,
    pub lights: Vec<Light>,
    pub planes: Vec<Plane<'a>>,
    pub spheres: Vec<Sphere<'a>>,
    pub triangles: Vec<Triangle<'a>>,

    pub meshes: Vec<Mesh<'a>>,
    pub mesh_instances: Vec<Instance<'a, 'a, Mesh<'a>>>,

    // TODO: why can't we do this?
    //     - Somehow Instances Primitive (dyn Intersect<_> here) size is implicitly required to be Sized.
    //     - Then, after a '+ ?Sized' bound is added to the Intersect definition,
    //       "`(dyn intersect::Intersect<Result = hitpoint::Hitpoint<'_>> + 'static)` cannot be shared between threads safely"
    //       appears because of not being Sync when used with rayon
    // pub dyn_instances: Vec<Box<Instance<'a, 'a, dyn Intersect<Result=Hitpoint<'a>>>>>,
    // TODO: why can't we do this?
    //     - `(dyn intersect::Intersect<Result = hitpoint::Hitpoint<'_>> + 'static)` cannot be shared between threads safely
    //       appears because of not being Sync when used with rayon
    // pub dyn_intersectables: Vec<Box<dyn Intersect<Result=Hitpoint<'a>> >>,

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

        let mut check_hitpoint =
            |hitpoint| utils::take_hitpoint_if_closer(&mut closest_hitpoint, hitpoint);

        for plane in &self.planes {
            check_hitpoint(plane.intersect(ray));
        }
        for sphere in &self.spheres {
            check_hitpoint(sphere.intersect(ray));
        }
        for triangle in &self.triangles {
            check_hitpoint(triangle.intersect(ray));
        }
        for mesh_instance in &self.mesh_instances {
            check_hitpoint(mesh_instance.intersect(ray));
        }

        closest_hitpoint
    }
}