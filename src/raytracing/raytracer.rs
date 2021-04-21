use crate::exercise1::Scene;
use crate::raytracing::transform::matrix;
use crate::raytracing::{Ray, Intersect};
use crate::raytracing::color::ColorRgb;

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