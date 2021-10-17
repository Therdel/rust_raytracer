#[cfg(test)]
mod tests {
    use num_traits::zero;
    use crate::raytracing::{Material, MaterialType};
    use crate::raytracing::{Intersect, Plane, Ray, Sphere, Triangle};
    use nalgebra_glm as glm;
    use crate::utils;

    fn test_material() -> Material {
        Material {
            name: String::from("marriage_material"),
            emissive: zero(),
            ambient: zero(),
            diffuse: zero(),
            specular: zero(),
            shininess: 0.0,
            material_type: MaterialType::Phong
        }
    }

    mod ray_sphere_intersection {
        use super::*;

        fn sphere(material: &Material) -> Sphere {
            Sphere {
                center: glm::vec3(0., 0., 0.),
                radius: 1.,
                material: material
            }
        }

        #[test]
        fn ray_origin_more_than_4096_times_r_units_away() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(0., 0., -4100.),
                direction: glm::vec3(0., 0., 1.),
            };

            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit sphere");
            utils::assert_approx_eq(hitpoint.t, 4099.);
        }

        #[test]
        fn ray_missing_sphere() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(0., 0., -2.),
                direction: glm::vec3(0., 1., 0.),
            };
            assert!(sphere.intersect(&ray).is_none());
        }

        #[test]
        fn ray_hits_sphere() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(0., 0., -2.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit sphere");

            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_inside_sphere() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(0., 0., 0.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit!");
            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_hits_sphere_tangentially() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(-1., 0., -1.),
                direction: glm::vec3(0., 0., 1.),
            };
            let hitpoint = sphere.intersect(&ray).expect("Ray didn't hit!");
            utils::assert_approx_eq(hitpoint.t, 1.0);
        }

        #[test]
        fn ray_points_away_from_sphere() {
            let material = test_material();
            let sphere = sphere(&material);

            let ray = Ray {
                origin: glm::vec3(0., 0., -1.1),
                direction: glm::vec3(0., 0., -1.),
            };
            let hitpoint = sphere.intersect(&ray);
            assert!(hitpoint.is_some() == false);
        }
    }
    mod ray_triangle_intersection {
        use super::*;

        fn triangle(material: &Material) -> Triangle {
            Triangle::new([glm::vec3(-1., 1., 0.),
                              glm::vec3(1., 0., 0.),
                              glm::vec3(-1., -1., 0.)],
                          [zero(); 3],
                          material)
        }

        #[test]
        fn ray_hits_triangle() {
            let material = test_material();
            let triangle = triangle(&material);

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., 1.)};
            let hitpoint = triangle.intersect(&ray).expect("Ray didn't hit triangle!");
            utils::assert_approx_eq(hitpoint.t, 2.);
        }

        #[test]
        fn ray_points_away_from_triangle() {
            let material = test_material();
            let triangle = triangle(&material);

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., -1.)};
            assert!(triangle.intersect(&ray).is_none());
        }

        #[test]
        fn ray_missing_triangle() {
            let material = test_material();
            let triangle = triangle(&material);

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 1., 0.)};
            assert!(triangle.intersect(&ray).is_none());
        }
    }

    mod ray_plane_intersection {
        use super::*;

        fn plane(material: &Material) -> Plane {
            Plane {
                normal: glm::vec3(0., 0., -1.),
                distance: 1.,
                material: &material
            }
        }

        #[test]
        fn ray_missing_plane() {
            let material = test_material();
            let plane = plane(&material);

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 1., 0.)};
            assert!(plane.intersect(&ray).is_none());
        }

        #[test]
        fn ray_hits_plane() {
            let material = test_material();
            let plane = plane(&material);

            let ray = Ray{origin: glm::vec3(0., 0., -2.), direction: glm::vec3(0., 0., 1.)};
            let hitpoint = plane.intersect(&ray).expect("Ray didn't hit plane!");
            utils::assert_approx_eq(hitpoint.t, 1.);
        }

        #[test]
        fn ray_starts_behind_plane() {
            let material = test_material();
            let plane = plane(&material);

            let ray = Ray{origin: glm::vec3(0., 0., 0.), direction: glm::vec3(0., 0., 1.)};
            assert!(plane.intersect(&ray).is_none());
        }
    }
}