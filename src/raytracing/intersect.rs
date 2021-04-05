use crate::raytracing::{Ray, Hitpoint, Sphere, Plane, Triangle};
use crate::utils;
use num_traits::identities::Zero;
use num_traits::Signed;

const NUMERIC_ERROR_COMPENSATION_OFFSET: f32 = 1e-5;

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
        let mut result = None;

        // m = rOrg - C
        let m = ray.origin - self.center;
        // a = rDir * rDir
        let a = glm::dot(ray.direction, ray.direction);
        // b = 2(m * rDir)
        let b = 2.0 * glm::dot(m, ray.direction);
        // c = (m * m) - r²
        let c = glm::dot(m, m) - glm::pow(self.radius, 2.0);

        // 4 rDir² (r² - (m - (m * rDir^ ) * rDir^ )² )
        // where rDir^ means normalized
        //
        // 4 dot(rDir, rDir)
        // * (pow(r, 2) - dot(m - dot(m, rDir^) * rDir^,
        //                    m - dot(m, rDir^) * rDir^)
        //   )
        let r_dir_norm = glm::normalize(ray.direction);
        let discriminant = 4.0 * glm::dot(ray.direction, ray.direction)
            * (glm::pow(self.radius, 2.0)
            - glm::dot(m - r_dir_norm * glm::dot(m, r_dir_norm),
                       m - r_dir_norm * glm::dot(m, r_dir_norm))
        );

        let t: Option<_> = if discriminant.is_zero() {
            Some((-0.5 * b ) / a)
        } else if discriminant.is_positive() {
            // calculate intersections
            // t0 = q / a
            // t1 = c / q
            //
            // where q = -0.5(b - sqrt(discriminant)  for b < 0
            //           -0.5(b + sqrt(discriminant)  otherwise
            let q;
            if b < 0.0 {
                q = -0.5 * (b - glm::sqrt(discriminant));
            } else {
                q = -0.5 * (b + glm::sqrt(discriminant));
            }
            let t0 = q / a;
            let t1 = c / q;


            if t0 < 0.0 && t1 >= 0.0 {
                // first intersection behind ray origin, second valid
                Some(t1)
            } else if t1 < 0.0 && t0 >= 0.0 {
                // second intersection behind ray origin, first falid
                Some(t0)
            } else {
                // either both behind ray origin (invalid) or both valid
                // take the first intersection in ray direction
                Some(glm::min(t0, t1))
            }
        } else {
            None
        };

        if let Some(t) = t {
            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);

                // compensate numeric error on intersection
                // move hitpoint along surface normal in direction of ray origin
                // this avoids cases where hitpoints numerically "sink through" the surface
                let normal = self.normal(&hit_position);
                let n_dot_rdir = glm::dot(normal, ray.direction);
                let intersect_frontside = n_dot_rdir < 0.0;
                let hit_normal = if intersect_frontside { normal } else { normal * -1.0 };
                let offset = hit_normal * NUMERIC_ERROR_COMPENSATION_OFFSET;

                let hitpoint = Hitpoint {t: t, position: hit_position + offset};
                result = Some(hitpoint);
            }
        }
        result
    }
}

impl Intersect for Plane {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        let mut result = None;

        let n_dot_rdir = glm::dot(self.normal, ray.direction);
        let parallel = n_dot_rdir == 0.0;
        if !parallel {
            // t = d - N * rOrg
            //     ------------
            //       N * rDir
            let t = (self.distance - glm::dot(self.normal, ray.origin))
                / n_dot_rdir;

            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);

                // compensate numeric error on intersection
                // move hitpoint along surface normal in direction of ray origin
                // this avoids cases where hitpoints numerically "sink through" the surface
                let intersect_frontside = n_dot_rdir < 0.0;
                let hit_normal = if intersect_frontside { self.normal } else { self.normal * -1.0 };
                let offset = hit_normal * NUMERIC_ERROR_COMPENSATION_OFFSET;

                let hitpoint = Hitpoint {t: t, position: hit_position + offset};
                result = Some(hitpoint);
            }
        }
        result
    }
}

impl Intersect for Triangle {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        todo!()
    }
}