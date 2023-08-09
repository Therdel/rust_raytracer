#[cfg(test)]
mod tests {
    use num_traits::zero;
    use crate::raytracing::{Intersect, MaterialIndex, Plane, Ray, Sphere, Triangle};
    use nalgebra_glm as glm;
    use crate::utils;

    mod ray_sphere_intersection {
        use super::*;

        fn sphere() -> Sphere {
            Sphere {
                center: glm::vec3(0., 0., 0.),
                radius: 1.,
                material: MaterialIndex(0)
            }
        }

        #[test]
        fn ray_origin_more_than_4096_times_r_units_away() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(0., 0., -4100.),
                direction: glm::vec3(0., 0., 1.),
            };

            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit sphere");
            utils::assert_approx_eq(hitpoint.t, 4099.);
        }

        #[test]
        fn ray_missing_sphere() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(0., 0., -2.),
                direction: glm::vec3(0., 1., 0.),
            };
            assert!(sphere.intersect(&ray).is_none());
        }

        #[test]
        fn ray_hits_sphere() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(0., 0., -2.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit sphere");

            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_inside_sphere() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(0., 0., 0.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit!");
            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_hits_sphere_tangentially() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(-1., 0., -1.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit!");
            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_points_away_from_sphere() {
            let sphere = sphere();

            let ray = Ray {
                origin: glm::vec3(0., 0., -1.1),
                direction: glm::vec3(0., 0., -1.),
            };
            let hitpoint = sphere.intersect(&ray);
            assert!(hitpoint.is_none());
        }
    }

    mod ray_triangle_intersection {
        use super::*;

        fn triangle() -> Triangle {
            Triangle::new([glm::vec3(-1., 1., 0.),
                              glm::vec3(1., 0., 0.),
                              glm::vec3(-1., -1., 0.)],
                          [zero(); 3],
                          MaterialIndex(0))
        }

        #[test]
        fn ray_hits_triangle() {
            let triangle = triangle();

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., 1.)};
            let hitpoint = triangle.intersect(&ray).expect("Ray didn't hit triangle!");
            utils::assert_approx_eq(hitpoint.t, 2.);
        }

        #[test]
        fn ray_points_away_from_triangle() {
            let triangle = triangle();

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., -1.)};
            assert!(triangle.intersect(&ray).is_none());
        }

        #[test]
        fn ray_missing_triangle() {
            let triangle = triangle();

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 1., 0.)};
            assert!(triangle.intersect(&ray).is_none());
        }
    }

    mod ray_plane_intersection {
        use super::*;

        fn plane() -> Plane {
            Plane::new(glm::vec3(0., 0., -1.), 1., MaterialIndex(0))
        }

        #[test]
        fn ray_missing_plane() {
            let plane = plane();

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 1., 0.)};
            assert!(plane.intersect(&ray).is_none());
        }

        #[test]
        fn ray_hits_plane() {
            let plane = plane();

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., 1.)};
            let hitpoint = plane.intersect(&ray).expect("Ray didn't hit plane!");
            utils::assert_approx_eq(hitpoint.t, 1.);
        }

        #[test]
        fn ray_starts_behind_plane() {
            let plane = plane();

            let ray = Ray{origin: glm::vec3(0., 0., 0.), direction: glm::vec3(0., 0., 1.)};
            assert!(plane.intersect(&ray).is_none());
        }
    }
}