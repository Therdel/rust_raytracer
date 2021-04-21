use crate::exercise1::Scene;
use crate::raytracing::transform::matrix;
use crate::raytracing::{Ray, Intersect, Light};
use crate::raytracing::color::ColorRgb;
use num_traits::Zero;

pub struct Raytracer<'scene> {
    scene: &'scene Scene<'scene>,
    screen_to_world: glm::Mat4
}

pub trait Public {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene>;

    fn depth_map(&self, ray: &Ray) -> Option<ColorRgb>;

    fn generate_primary_ray(&self, screen_coordinate: &glm::Vec2) -> Ray;
}

trait Private {
    fn trace_shadow_ray(&self, world_pos: &glm::Vec3, light: &Light) -> bool;
}

impl Public for Raytracer<'_> {
    fn new<'scene>(scene: &'scene Scene) -> Raytracer<'scene> {
        Raytracer {
            scene,
            screen_to_world: matrix::screen_to_world(&scene.camera),
        }
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