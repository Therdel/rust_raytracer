use crate::raytracing::{Ray, Hitpoint, Sphere, Plane, Triangle, Material, AABB, Instance, Mesh};
use crate::utils;
use nalgebra_glm as glm;
use num_traits::identities::Zero;
use num_traits::Signed;
use crate::utils::AliasRc;

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
        let a = glm::dot(&ray.direction, &ray.direction);
        // b = 2(m * rDir)
        let b = 2.0 * glm::dot(&m, &ray.direction);
        // c = (m * m) - r²
        let c = glm::dot(&m, &m) - self.radius*self.radius;

        // 4 rDir² (r² - (m - (m * rDir^ ) * rDir^ )² )
        // where rDir^ means normalized
        //
        // 4 dot(rDir, rDir)
        // * (pow(r, 2) - dot(m - dot(m, rDir^) * rDir^,
        //                    m - dot(m, rDir^) * rDir^)
        //   )
        let r_dir_norm = glm::normalize(&ray.direction);
        let discriminant = 4.0 * glm::dot(&ray.direction, &ray.direction)
            * (self.radius*self.radius
            - glm::dot(&(m - r_dir_norm * glm::dot(&m, &r_dir_norm)),
                       &(m - r_dir_norm * glm::dot(&m, &r_dir_norm)))
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
                q = -0.5 * (b - discriminant.sqrt());
            } else {
                q = -0.5 * (b + discriminant.sqrt());
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
                Some(f32::min(t0, t1))
            }
        } else {
            None
        };

        if let Some(t) = t {
            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);
                let normal = self.normal(&hit_position);
                let hitpoint = create_hitpoint(t, &hit_position, ray, &normal, self.material.clone());

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

        let n_dot_rdir = glm::dot(&self.normal, &ray.direction);
        let parallel = n_dot_rdir == 0.0;
        if !parallel {
            // t = d - N * rOrg
            //     ------------
            //       N * rDir
            let t = (self.distance - glm::dot(&self.normal, &ray.origin))
                / n_dot_rdir;

            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);
                let hitpoint = create_hitpoint(t, &hit_position, ray, &self.normal, self.material.clone());

                result = Some(hitpoint);
            }
        }
        result
    }
}

impl Intersect for Triangle {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        let mut result = None;
        let (a, b, c) = (&self.vertices[0], &self.vertices[1], &self.vertices[2]);

        let e1 = *b - *a;
        let e2 = *c - *a;
        let s = ray.origin - *a;
        let q = glm::cross(&ray.direction, &e2);
        let r = glm::cross(&s, &e1);

        let q_dot_e1 = glm::dot(&q, &e1);

        let t = glm::dot(&r, &e2) / q_dot_e1;
        let v = glm::dot(&q, &s) / q_dot_e1;
        let w = glm::dot(&r, &ray.direction) / q_dot_e1;
        let u = 1.0 - v - w;

        let is_ray_parallel = glm::dot(&e1, &q) == 0.0;
        // TODO: Document that the official solution (e1 * q) < 0 discards intersections from behind the triangle.
        let does_ray_point_away = t < 0.0; //glm::dot(e1, q) < 0; // FIXME: Why not [..] = t < 0 ?
        let is_hit_point_outside = u < 0.0 || v < 0.0 || u + v > 1.0;

        let does_intersect = !is_ray_parallel &&
            !does_ray_point_away &&
            !is_hit_point_outside;
        if does_intersect {
            let hit_position = utils::ray_equation(ray, t);
            let hitpoint = create_hitpoint(t, &hit_position, ray, self.normal(), self.material.clone());

            result = Some(hitpoint);
        }

        result
    }
}

impl Intersect for AABB {
    type Result = ();

    // source: https://gamedev.stackexchange.com/a/18459
    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let dirfrac = glm::vec3(
            // r.dir is unit direction vector of ray
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        // lb is the corner of AABB with minimal coordinates - left bottom, rt is maximal corner
        // r.org is origin of ray
        let lb = &self.min;
        let rt = &self.max;
        let t1 = (lb.x - ray.origin.x)*dirfrac.x;
        let t2 = (rt.x - ray.origin.x)*dirfrac.x;
        let t3 = (lb.y - ray.origin.y)*dirfrac.y;
        let t4 = (rt.y - ray.origin.y)*dirfrac.y;
        let t5 = (lb.z - ray.origin.z)*dirfrac.z;
        let t6 = (rt.z - ray.origin.z)*dirfrac.z;

        let tmin = f32::max(f32::max(f32::min(t1, t2), f32::min(t3, t4)), f32::min(t5, t6));
        let tmax = f32::min(f32::min(f32::max(t1, t2), f32::max(t3, t4)), f32::max(t5, t6));

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        let _t;
        if tmax < 0.0 {
            _t = tmax;
            return None;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            _t = tmax;
            return None;
        }

        _t = tmin;
        Some(())
    }
}

impl Intersect for Mesh {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let mut closest_hitpoint = None;

        let mut check_hitpoint =
            |hitpoint| utils::take_hitpoint_if_closer(&mut closest_hitpoint, hitpoint);

        for triangle in &self.triangles {
            check_hitpoint(triangle.intersect(ray));
        }

        closest_hitpoint
    }
}

impl<Primitive> Intersect for Instance<Primitive>
where Primitive: Intersect<Result=Hitpoint> {
    type Result = Primitive::Result;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result>{
        let transform = |vec: &glm::Vec3, mat: &glm::Mat4| -> glm::Vec3 {
            let homogeneous_transformed = *mat * vec.push(1.0);
            // no perspective divide needed as we're only using translate, scale & rotate
            homogeneous_transformed.xyz()
        };

        // transform ray into model-local coordinate-system
        let transformed_origin = transform(&ray.origin, &self.model_inverse);
        let transformed_direction = glm::normalize(
            &transform(&ray.direction, &self.rotation_scale_inverse)
        );
        let transformed_ray = Ray { origin: transformed_origin, direction: transformed_direction };

        let mut hitpoint = self.primitive.intersect(&transformed_ray)?;
        // transform hitpoint back into world-local coordinate-system
        hitpoint.position = transform(&hitpoint.position, &self.model);
        hitpoint.hit_normal = glm::normalize(
            &transform(&hitpoint.hit_normal, &self.rotation_scale)
        );
        hitpoint.position_for_refraction = transform(&hitpoint.position_for_refraction, &self.model);

        let t_in_world = glm::distance(&ray.origin, &hitpoint.position);
        hitpoint.t = t_in_world;

        // TODO: Why does this work with both ```ref material``` and ```material```?
        if let Some(ref material) = self.material_override {
            hitpoint.material = material.clone();
        }
        Some(hitpoint)
    }
}

fn create_hitpoint(t: f32, hit_position: &glm::Vec3, ray: &Ray, normal: &glm::Vec3,
                   material: AliasRc<Vec<Material>, Material>) -> Hitpoint {
    let n_dot_rdir = glm::dot(normal, &ray.direction);
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