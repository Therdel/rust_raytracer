use crate::raytracing::{self, Ray, Hitpoint, Sphere, Plane, Triangle, Light, Camera, Material, Mesh, Screen, Instance};
use crate::utils;

pub struct Scene {
    pub camera: Camera,
    pub screen: Screen,
    pub lights: Vec<Light>,
    pub materials: Vec<Material>,

    pub planes: Vec<Plane>,
    pub spheres: Vec<Sphere>,
    pub triangles: Vec<Triangle>,
    pub meshes: Vec<Mesh>,
    pub mesh_instances: Vec<Instance<Mesh>>,
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

        check_hitpoint(self.planes.as_slice().intersect(ray));
        check_hitpoint(self.spheres.as_slice().intersect(ray));
        check_hitpoint(self.triangles.as_slice().intersect(ray));
        self.mesh_instances.iter()
            .zip(std::iter::repeat(self.meshes.as_slice()))
            .map(|tuple| tuple.intersect(ray))
            .for_each(check_hitpoint);

        closest_hitpoint
    }
}