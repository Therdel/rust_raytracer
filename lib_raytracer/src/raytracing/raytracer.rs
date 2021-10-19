use crate::exercise1::Scene;
use crate::raytracing::transform::matrix;
use crate::raytracing::{Ray, Intersect, Light, Hitpoint, MaterialType};
use crate::raytracing::color::{ColorRgb, self};
use nalgebra_glm as glm;
use num_traits::{Zero, zero};

const MAX_RAY_RECURSION_DEPTH: usize = 10;
const REFLECTION_DIM_FACTOR: f32 = 0.8;

pub struct Raytracer<'scene> {
    scene: &'scene Scene,
    screen_to_world: glm::Mat4
}

pub trait Public {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene>;

    fn raytrace(&self, ray: &Ray) -> Option<ColorRgb>;
    fn depth_map(&self, ray: &Ray) -> Option<ColorRgb>;

    fn generate_primary_ray(&self, screen_coordinate: &glm::Vec2) -> Ray;
}

trait Private {
    fn raytrace_impl(&self, ray: &Ray, ray_recursion_depth: usize) -> Option<ColorRgb>;
    fn shade(&self, ray: &Ray, hitpoint: &Hitpoint, ray_recursion_depth: usize) -> Option<ColorRgb>;
    fn radiance(&self, ray: &Ray, hitpoint: &Hitpoint, light: &Light, is_shadow: bool) -> ColorRgb;
    fn trace_shadow_ray(&self, world_pos: &glm::Vec3, light: &Light) -> bool;
    fn create_reflected_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3) -> glm::Vec3;
    fn create_transmitted_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3,
                              n1_current: f32, n2_pierce: f32) -> glm::Vec3;
    fn get_hitpoint_to_light_unit_vector(hitpoint: &Hitpoint, light: &Light) -> glm::Vec3;
    fn get_fresnel_factor(reflected_ray: &Ray, transmitted_ray: &Ray,
                          reflection_normal: &glm::Vec3, n1_current: f32, n2_pierce: f32) -> f32;
}

impl Public for Raytracer<'_> {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene> {
        Raytracer {
            scene,
            screen_to_world: matrix::screen_to_world(&scene.camera, &scene.screen),
        }
    }

    fn raytrace(&self, ray: &Ray) -> Option<ColorRgb> {
        self.raytrace_impl(ray, 0)
    }

    fn depth_map(&self, ray: &Ray) -> Option<ColorRgb> {
        let hitpoint = self.scene.intersect(&ray)?;

        let scale = 1.0 / 10.0;
        let brightness = hitpoint.t * scale;
        let color = glm::vec3(brightness, brightness, brightness);

        Some(color)
    }

    fn generate_primary_ray(&self, screen_coordinate: &glm::Vec2) -> Ray {
        let p_screen = glm::vec4(screen_coordinate.x, screen_coordinate.y, 0.0, 1.0);
        // TODO: Document that NDC "looks" in *positive* z-axis. Document wrong viewing direction
        //       Erklärung: Hat was mit der z-Range zutun, wie man die definiert.
        // TODO: Document that this is *always* in camera view direction. (NDC)
        let p_screen_forward = p_screen + glm::vec4(0.0, 0.0, 1.0, 0.0);

        let p_world = self.screen_to_world * p_screen;
        let p_world_forward = self.screen_to_world * p_screen_forward;

        let p_world_inhomogeneous = (p_world / p_world.w).xyz();
        let p_world_forward_inhomogeneous = (p_world_forward / p_world_forward.w).xyz();

        let direction = p_world_forward_inhomogeneous - p_world_inhomogeneous;
        let direction_normalized = glm::normalize(&direction);

        Ray {
            origin: p_world_inhomogeneous,
            direction: direction_normalized,
        }
    }
}

impl Private for Raytracer<'_> {
    fn raytrace_impl(&self, ray: &Ray, ray_recursion_depth: usize) -> Option<ColorRgb> {
        if ray_recursion_depth < MAX_RAY_RECURSION_DEPTH {
            let hitpoint = self.scene.intersect(ray)?;
            self.shade(ray, &hitpoint, ray_recursion_depth)
        } else {
            None
        }
    }

    fn shade(&self, ray: &Ray, hitpoint: &Hitpoint, ray_recursion_depth: usize) -> Option<ColorRgb> {
        let shade_phong = || {
            let mut current_color = None;
            for light in self.scene.lights.iter() {
                let is_shadow = self.trace_shadow_ray(&hitpoint.position, light);
                let radiance_color = self.radiance(ray, hitpoint, light, is_shadow);

                current_color = color::add_option(current_color, Some(radiance_color));
            }
            current_color
        };

        let shade_reflect = || {
            // TODO: why can't we do ```-&ray.direction``` here?
            let direction = Self::create_reflected_ray(&-ray.direction, &hitpoint.hit_normal);
            let reflected_ray = Ray { origin: hitpoint.position, direction: glm::normalize(&direction), };

            let reflection_color =
                self.raytrace_impl(&reflected_ray, ray_recursion_depth + 1)
                    .unwrap_or(self.scene.screen.background);
            Some(reflection_color * REFLECTION_DIM_FACTOR)
        };

        let shade_refract = |index_inner: f32, index_outer: f32| -> Option<ColorRgb> {
            let on_frontside = hitpoint.on_frontside;
            let origin_transmitted = &hitpoint.position_for_refraction;
            let (n1_current, n2_pierce) = match on_frontside {
                true => (index_outer, index_inner),
                false => (index_inner, index_outer)
            };
            let direction_transmitted = Self::create_transmitted_ray(&-ray.direction, &hitpoint.hit_normal, n1_current, n2_pierce);

            let transmitted_ray = Ray { origin: *origin_transmitted, direction: glm::normalize(&direction_transmitted)};
            let direction_reflected = Self::create_reflected_ray(&-ray.direction, &hitpoint.hit_normal);
            let reflected_ray = Ray { origin: hitpoint.position, direction: glm::normalize(&direction_reflected) };

            let reflected_color = self.raytrace_impl(&reflected_ray, ray_recursion_depth + 1)
                .unwrap_or(self.scene.screen.background);
            let transmitted_color = self.raytrace_impl(&transmitted_ray, ray_recursion_depth + 1)
                .unwrap_or(self.scene.screen.background);

            let k_reflected = Self::get_fresnel_factor(&reflected_ray, &transmitted_ray,
                                                       &hitpoint.hit_normal,
                                                       n1_current, n2_pierce);
            let k_transmitted = 1.0 - k_reflected;

            let color = reflected_color * k_reflected +
                transmitted_color * k_transmitted;

            Some(color)
        };

        match &hitpoint.material.material_type {
            &MaterialType::Phong => shade_phong(),
            &MaterialType::ReflectAndPhong => color::add_option(shade_reflect(), shade_phong()),
            &MaterialType::ReflectAndRefract {
                index_inner,
                index_outer
            } => shade_refract(index_inner, index_outer)
        }
    }

    fn radiance(&self, ray: &Ray, hitpoint: &Hitpoint, light: &Light, is_shadow: bool) -> ColorRgb {
        let l = Self::get_hitpoint_to_light_unit_vector(hitpoint, light);
        let n = hitpoint.hit_normal;
        let v = -ray.direction;
        let r = Self::create_reflected_ray(&l, &n);

        let l_dot_n = f32::max(glm::dot(&l,&n), 0.0);
        let r_dot_v = f32::max(glm::dot(&r, &v), 0.0);

        let emissive = hitpoint.material.emissive;
        let ambient = light.color.ambient + hitpoint.material.ambient;
        let diffuse = if is_shadow { zero() } else { light.color.diffuse.component_mul(&hitpoint.material.diffuse) * l_dot_n };
        let specular = if is_shadow { zero() } else { light.color.specular.component_mul(&hitpoint.material.specular) * r_dot_v.powf(hitpoint.material.shininess) };

        let radiance = emissive + ambient + diffuse + specular;
        radiance
    }

    fn trace_shadow_ray(&self, world_pos: &glm::Vec3, light: &Light) -> bool {
        let is_directional_light = light.position.w.is_zero();

        let direction = {
            if is_directional_light {
                light.position.xyz()
            } else {
                let light_world_pos = (light.position / light.position.w).xyz();
                light_world_pos - *world_pos
            }
        };

        let direction = glm::normalize(&direction);

        let ray = Ray { origin: *world_pos, direction };

        let is_shadow;
        if let Some(hitpoint) = self.scene.intersect(&ray) {
            if is_directional_light {
                // any intersection puts shadow of infinitely distant (directional light)
                is_shadow = true;
            } else {
                let light_world_pos = (light.position / light.position.w).xyz();
                let distance_to_light = glm::distance(&ray.origin, &light_world_pos);
                let ray_distance_travelled = hitpoint.t;

                is_shadow = ray_distance_travelled < distance_to_light;
            }
        } else {
            is_shadow = false;
        }

        is_shadow
    }

    fn create_reflected_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3) -> glm::Vec3 {
        *normal * 2. * (glm::dot(normal, to_viewer)) - *to_viewer
    }

    fn create_transmitted_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3,
                              n1_current: f32, n2_pierce: f32) -> glm::Vec3 {
        // TODO: Check using rust safe math
        let n = n1_current / n2_pierce;
        let w = n * glm::dot(to_viewer, normal);
        let k = f32::sqrt(1.0 + (w-n)*(w+n));
        let t = *normal * (w-k)  - *to_viewer * n;
        t
    }

    fn get_hitpoint_to_light_unit_vector(hitpoint: &Hitpoint, light: &Light) -> glm::Vec3 {
        let is_directional_light = light.position.w.is_zero();
        let vector = {
            if is_directional_light {
                light.position.xyz()
            } else {
                let light_world_pos = (light.position / light.position.w).xyz();
                light_world_pos - hitpoint.position
            }
        };
        glm::normalize(&vector)
    }

    fn get_fresnel_factor(reflected_ray: &Ray, transmitted_ray: &Ray,
                          reflection_normal: &glm::Vec3,
                          n1_current: f32, n2_pierce: f32) -> f32 {
        let transmission_normal = -*reflection_normal;


        // cos(ang) = (a dot b) / (len(a) *len(b))
        // both vectors are unit vectors, therefore only the dot product is needed
        let cos_ang_i = glm::dot(&reflected_ray.direction, reflection_normal);
        let cos_ang_t = glm::dot(&transmitted_ray.direction, &transmission_normal);

        let n_i = n1_current;
        let n_t = n2_pierce;

        let r_parallel = (n_t * cos_ang_i - n_i * cos_ang_t) / (n_t * cos_ang_i + n_i * cos_ang_t);
        let r_orthogonal = (n_i * cos_ang_i - n_t * cos_ang_t) / (n_i * cos_ang_i + n_t * cos_ang_t);

        let fresnel_factor_reflection = 0.5 * (r_parallel*r_parallel + r_orthogonal*r_orthogonal);
        fresnel_factor_reflection
    }
}

// TODO: Why does the second one compile but the first doesn't?
// impl<'scene> Public for Raytracer<'_> {
//    fn new(scene: &'scene Scene) -> Raytracer<'scene> {
//         let screen_to_world = crate::raytracing::transform::matrix::screen_to_world(&scene.camera);
//         Raytracer {
//             scene,
//             screen_to_world,
//         }
//     }
// }
// impl Public for Raytracer<'_> {
//     fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene> {
//         let screen_to_world = crate::raytracing::transform::matrix::screen_to_world(&scene.camera);
//         Raytracer {
//             scene,
//             screen_to_world,
//         }
//     }
// }