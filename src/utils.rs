use crate::raytracing::Ray;

#[cfg(test)]
pub fn assert_approx_eq(a: f32, b: f32) {
    float_eq::assert_float_eq!(a, b, rmax <= 2.0 * f32::EPSILON)
}

pub fn ray_equation(ray: &Ray, t: f32) -> glm::Vec3 {
    ray.origin + ray.direction * t
}

pub type ColorRGB = glm::Vec3;