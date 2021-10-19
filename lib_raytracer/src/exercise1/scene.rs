use crate::raytracing::{self, Ray, Hitpoint, Sphere, Plane, Triangle, Light, Camera, Material, Instance, Mesh, Screen};
use crate::raytracing::color::ColorRgb;
use crate::utils;
use crate::utils::AliasArc;

pub struct Scene {
    pub camera: Camera,
    pub screen: Screen,
    pub lights: AliasArc<Vec<Light>, [Light]>,
    pub materials: AliasArc<Vec<Material>, [Material]>,

    pub planes: AliasArc<Vec<Plane>, [Plane]>,
    pub spheres: AliasArc<Vec<Sphere>, [Sphere]>,
    pub triangles: AliasArc<Vec<Triangle>, [Triangle]>,
    pub meshes: AliasArc<Vec<Mesh>, [Mesh]>,
    pub mesh_instances: AliasArc<Vec<Instance<Mesh>>, [Instance<Mesh>]>,

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

impl raytracing::Intersect for Scene {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let mut closest_hitpoint: Option<Self::Result> = None;

        let mut check_hitpoint =
            |hitpoint| utils::take_hitpoint_if_closer(&mut closest_hitpoint, hitpoint);

        for plane in self.planes.iter() {
            check_hitpoint(plane.intersect(ray));
        }
        for sphere in self.spheres.iter() {
            check_hitpoint(sphere.intersect(ray));
        }
        for triangle in self.triangles.iter() {
            check_hitpoint(triangle.intersect(ray));
        }
        for mesh_instance in self.mesh_instances.iter() {
            check_hitpoint(mesh_instance.intersect(ray));
        }

        closest_hitpoint
    }
}