use nalgebra_glm as glm;
use crate::raytracing::transform::matrix;
use crate::raytracing::{Background, self, Ray, Hitpoint, Sphere, Plane, Triangle, Light, Camera, Material, Mesh, Instance};
use crate::utils;

pub struct Scene {
    camera: Camera,
    screen_to_world: glm::Mat4,
    pub background: Background,
    pub lights: Vec<Light>,
    pub materials: Vec<Material>,

    pub planes: Vec<Plane>,
    pub spheres: Vec<Sphere>,
    pub triangles: Vec<Triangle>,
    pub meshes: Vec<Mesh>,
    pub mesh_instances: Vec<Instance<Mesh>>,
}

impl Scene {
    pub fn new(camera: Camera, background: Background) -> Self {
        let screen_to_world = matrix::screen_to_world(&camera);
        Self {
            camera,
            screen_to_world,
            background,
            lights: vec![],
            materials: vec![],
            planes: vec![],
            spheres: vec![],
            triangles: vec![],
            meshes: vec![],
            mesh_instances: vec![],
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn update_camera(&mut self, f: impl Fn(&mut Camera)) {
        f(&mut self.camera);
        self.screen_to_world = matrix::screen_to_world(&self.camera);
    }

    pub fn screen_to_world(&self) -> &glm::Mat4 {
        &self.screen_to_world
    }

    pub fn resize_screen(&mut self, width: usize, height: usize) {
        self.camera.screen_dimensions = glm::vec2(width as _, height as _);
    
        self.screen_to_world = matrix::screen_to_world(&self.camera);
    }

    pub fn turn_camera(&mut self, begin: &glm::Vec2, end: &glm::Vec2) {
        let radians = |degrees: f32| degrees * (glm::pi::<f32>() / 180.0);
    
        // pixel to degrees mapping
        let y_fov_degrees = self.camera.y_fov_degrees;
        let degrees_per_pixel = y_fov_degrees / self.camera.screen_dimensions.y as f32;
        let pixel_to_angle = |pixel| radians(pixel * degrees_per_pixel);
    
        let pixel_diff_x = end.x - begin.x;
        let pixel_diff_y = end.y - begin.y;
    
        let angle_diff_heading = pixel_to_angle(pixel_diff_x);
        let angle_diff_pitch = pixel_to_angle(pixel_diff_y);
    
        // "natural scrolling" - turning follows the inverse cursor motion
        // the heading turn is positive when turning to the left -> when drag_begin is left of drag_end
        let angle_diff_heading = match begin.x < end.x {
            true => angle_diff_heading.abs(),
            false => -angle_diff_heading.abs()
        };
        // the pitch turn is positive when turning upwards -> when drag_begin is above drag_end
        let angle_diff_pitch = match begin.y > end.y {
            true => angle_diff_pitch.abs(),
            false => -angle_diff_pitch.abs()
        };
    
        let camera_orientation = &mut self.camera.orientation;
        camera_orientation.x += angle_diff_pitch;
        camera_orientation.y += angle_diff_heading;
    
        // clamp pitch
        camera_orientation.x = camera_orientation.x.clamp(radians(-90.),
                                                          radians(90.));
        // modulo heading
        camera_orientation.y %= radians(360.);
    
        self.screen_to_world = matrix::screen_to_world(&self.camera);
    }
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