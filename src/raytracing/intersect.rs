use crate::raytracing::{Ray, Hitpoint, Sphere, Plane, Triangle, Material};
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

impl<'a> Intersect for Sphere<'a> {
    type Result = Hitpoint<'a>;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint<'a>> {
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
                let normal = self.normal(&hit_position);
                let hitpoint = create_hitpoint(t, &hit_position, ray, &normal, self.material);

                result = Some(hitpoint);
            }
        }
        result
    }
}

impl<'a> Intersect for Plane<'a> {
    type Result = Hitpoint<'a>;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint<'a>> {
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
                let hitpoint = create_hitpoint(t, &hit_position, ray, &self.normal, self.material);

                result = Some(hitpoint);
            }
        }
        result
    }
}

impl<'a> Intersect for Triangle<'a> {
    type Result = Hitpoint<'a>;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint<'a>> {
        let mut result = None;

        let e1 = self.b - self.a;
        let e2 = self.c - self.a;
        let s = ray.origin - self.a;
        let q = glm::cross(ray.direction, e2);
        let r = glm::cross(s, e1);

        let q_dot_e1 = glm::dot(q, e1);

        let t = glm::dot(r, e2) / q_dot_e1;
        let v = glm::dot(q, s) / q_dot_e1;
        let w = glm::dot(r, ray.direction) / q_dot_e1;
        let u = 1.0 - v - w;

        let is_ray_parallel = glm::dot(e1, q) == 0.0;
        // TODO: Document that the official solution (e1 * q) < 0 discards intersections from behind the triangle.
        let does_ray_point_away = t < 0.0; //glm::dot(e1, q) < 0; // FIXME: Why not [..] = t < 0 ?
        let is_hit_point_outside = u < 0.0 || v < 0.0 || u + v > 1.0;

        let does_intersect = !is_ray_parallel &&
            !does_ray_point_away &&
            !is_hit_point_outside;
        if does_intersect {
            let hit_position = utils::ray_equation(ray, t);
            let hitpoint = create_hitpoint(t, &hit_position, ray, self.normal(), self.material);

            result = Some(hitpoint);
        }

        result
    }
}

fn create_hitpoint<'material>(t: f32, hit_position: &glm::Vec3, ray: &Ray, normal: &glm::Vec3, material: &'material Material) -> Hitpoint<'material> {
    let n_dot_rdir = glm::dot(*normal, ray.direction);
    let intersect_frontside = n_dot_rdir < 0.0;

    // invert surface normal when hitting the back or inside of the geometry
    let hit_normal = if intersect_frontside { *normal } else { -*normal };

    // compensate numeric error on intersection.
    // moves hitpoint along surface normal in direction of ray origin
    // this avoids cases where hitpoints numerically "sink through" the surface
    let offset = hit_normal * NUMERIC_ERROR_COMPENSATION_OFFSET;
    let hit_position_acne_compensated = *hit_position + offset;

    // refractive ray begins on the other side of the geometry.
    // Preventing acne effects on this side, the acne compensation vector is
    // substracted from the hit position
    let hit_position_for_refraction = *hit_position - offset;

    Hitpoint {
        t,
        position: hit_position_acne_compensated,
        hit_normal,
        position_for_refraction: hit_position_for_refraction,
        on_frontside: intersect_frontside,
        material: material,
    }
}