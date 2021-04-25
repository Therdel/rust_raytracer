use crate::exercise1::Scene;
use crate::raytracing::transform::matrix;
use crate::raytracing::{Ray, Intersect, Light, Hitpoint};
use crate::raytracing::color::ColorRgb;
use num_traits::{Zero, zero};

pub struct Raytracer<'scene> {
    scene: &'scene Scene<'scene>,
    screen_to_world: glm::Mat4
}

pub trait Public {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene>;

    fn raytrace(&self, ray: &Ray) -> Option<ColorRgb>;
    fn depth_map(&self, ray: &Ray) -> Option<ColorRgb>;

    fn generate_primary_ray(&self, screen_coordinate: &glm::Vec2) -> Ray;
}

trait Private {
    fn shade(&self, ray: &Ray, hitpoint: &Hitpoint) -> Option<ColorRgb>;
    fn radiance(&self, ray: &Ray, hitpoint: &Hitpoint, light: &Light, is_shadow: bool) -> ColorRgb;
    fn trace_shadow_ray(&self, world_pos: &glm::Vec3, light: &Light) -> bool;
    fn create_reflected_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3) -> glm::Vec3;
    fn get_hitpoint_to_light_unit_vector(hitpoint: &Hitpoint, light: &Light) -> glm::Vec3;
}

impl Public for Raytracer<'_> {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene> {
        Raytracer {
            scene,
            screen_to_world: matrix::screen_to_world(&scene.camera),
        }
    }

    fn raytrace(&self, ray: &Ray) -> Option<ColorRgb> {
        let hitpoint = self.scene.intersect(ray)?;
        self.shade(ray, &hitpoint)
    }

    fn depth_map(&self, ray: &Ray) -> Option<ColorRgb> {
        let hitpoint = self.scene.intersect(&ray)?;

        let scale = 1.0 / 10.0;
        let brightness = hitpoint.t * scale;
        let color = glm::vec3(brightness, brightness, brightness);

        Some(color)
    }

    fn generate_primary_ray(&self, screen_coordinate: &glm::Vec2) -> Ray {
        screen_coordinate.extend(0.0).extend(1.);
        let p_screen = glm::vec4(screen_coordinate.x, screen_coordinate.y, 0.0, 1.0);
        // TODO: Document that NDC "looks" in *positive* z-axis. Document wrong viewing direction
        //       Erklärung: Hat was mit der z-Range zutun, wie man die definiert.
        // TODO: Document that this is *always* in camera view direction. (NDC)
        let p_screen_forward = p_screen + glm::vec4(0.0, 0.0, 1.0, 0.0);

        let p_world = self.screen_to_world * p_screen;
        let p_world_forward = self.screen_to_world * p_screen_forward;

        let p_world_inhomogeneous = (p_world / p_world.w).truncate(3);
        let p_world_forward_inhomogeneous = (p_world_forward / p_world_forward.w).truncate(3);

        let direction = p_world_forward_inhomogeneous - p_world_inhomogeneous;
        let direction_normalized = glm::normalize(direction);

        Ray {
            origin: p_world_inhomogeneous,
            direction: direction_normalized,
        }
    }
}

impl Private for Raytracer<'_> {
    fn shade(&self, ray: &Ray, hitpoint: &Hitpoint) -> Option<ColorRgb> {
        let mut color = None;

        for light in &self.scene.lights {
            let is_shadow = self.trace_shadow_ray(&hitpoint.position, light);
            let radiance_color = self.radiance(ray, hitpoint, light, is_shadow);

            match color {
                Some(old_color) =>
                    color = Some(old_color + radiance_color),
                None =>
                    color = Some(radiance_color)
            }
        }

        color
    }

    fn radiance(&self, ray: &Ray, hitpoint: &Hitpoint, light: &Light, is_shadow: bool) -> ColorRgb {
        let l = Self::get_hitpoint_to_light_unit_vector(hitpoint, light);
        let n = hitpoint.hit_normal;
        let v = -ray.direction;
        let r = Self::create_reflected_ray(&l, &n);

        let l_dot_n = glm::max(glm::dot(l, n), 0.0);
        let r_dot_v = glm::max(glm::dot(r, v), 0.0);

        let emissive = hitpoint.material.emissive;
        let ambient = light.color.ambient + hitpoint.material.ambient;
        let diffuse = if is_shadow { zero() } else { light.color.diffuse * hitpoint.material.diffuse * l_dot_n };
        let specular = if is_shadow { zero() } else { light.color.specular * hitpoint.material.specular * glm::pow(r_dot_v, hitpoint.material.shininess) };

        let radiance = emissive + ambient + diffuse + specular;
        radiance
    }

    fn trace_shadow_ray(&self, world_pos: &glm::Vec3, light: &Light) -> bool {
        let is_directional_light = light.position.w.is_zero();

        let direction = {
            if is_directional_light {
                light.position.truncate(3)
            } else {
                let light_world_pos = (light.position / light.position.w).truncate(3);
                light_world_pos - *world_pos
            }
        };

        let direction = glm::normalize(direction);

        let ray = Ray { origin: *world_pos, direction };

        let is_shadow;
        if let Some(hitpoint) = self.scene.intersect(&ray) {
            if is_directional_light {
                // any intersection puts shadow of infinitely distant (directional light)
                is_shadow = true;
            } else {
                let light_world_pos = (light.position / light.position.w).truncate(3);
                let distance_to_light = glm::distance(ray.origin, light_world_pos);
                let ray_distance_travelled = hitpoint.t;

                is_shadow = ray_distance_travelled < distance_to_light;
            }
        } else {
            is_shadow = false;
        }

        is_shadow
    }

    fn create_reflected_ray(to_viewer: &glm::Vec3, normal: &glm::Vec3) -> glm::Vec3 {
        *normal * 2. * (glm::dot(*normal, *to_viewer)) - *to_viewer
    }

    fn get_hitpoint_to_light_unit_vector(hitpoint: &Hitpoint, light: &Light) -> glm::Vec3 {
        let is_directional_light = light.position.w.is_zero();
        let vector = {
            if is_directional_light {
                light.position.truncate(3)
            } else {
                let light_world_pos = (light.position / light.position.w).truncate(3);
                light_world_pos - hitpoint.position
            }
        };
        glm::normalize(vector)
    }
}

/// TODO: Why does the second one compile but the first doesn't?
/// ```
/// impl<'scene> Public for Raytracer<'_> {
///    fn new(scene: &'scene Scene) -> Raytracer<'scene> {
///         let screen_to_world = crate::raytracing::transform::matrix::screen_to_world(&scene.camera);
///         Raytracer {
///             scene,
///             screen_to_world,
///         }
///     }
/// }
/// impl Public for Raytracer<'_> {
///     fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene> {
///         let screen_to_world = crate::raytracing::transform::matrix::screen_to_world(&scene.camera);
///         Raytracer {
///             scene,
///             screen_to_world,
///         }
///     }
/// }```
const _TODO_MESSAGE: () = ();