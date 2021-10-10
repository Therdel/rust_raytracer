use crate::raytracing::{Ray, Hitpoint};
use nalgebra_glm as glm;

#[cfg(test)]
pub fn assert_approx_eq(a: f32, b: f32) {
    float_eq::assert_float_eq!(a, b, rmax <= 2.0 * f32::EPSILON)
}

pub fn ray_equation(ray: &Ray, t: f32) -> glm::Vec3 {
    ray.origin + ray.direction * t
}

pub fn take_hitpoint_if_closer<'material>(closest_hitpoint: &mut Option<Hitpoint<'material>>,
                                          hitpoint: Option<Hitpoint<'material>>) {
    if let Some(hitpoint) = hitpoint {
        if let Some(ref mut closest_hitpoint) = closest_hitpoint {
            if hitpoint.t < closest_hitpoint.t {
                *closest_hitpoint = hitpoint;
            }
        } else {
            *closest_hitpoint = Some(hitpoint);
        }
    }
}