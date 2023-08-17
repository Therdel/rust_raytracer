pub mod matrix {
    use nalgebra_glm as glm;
    use num_traits::one;
    use crate::raytracing::Camera;

    pub fn translation(offset: &glm::Vec3) -> glm::Mat4 {
        glm::translation(offset)
    }

    pub fn scaling(scale: &glm::Vec3) -> glm::Mat4 {
        glm::scaling(scale)
    }

    pub fn rotation(yaw: f32,
                    pitch: f32,
                    roll: f32) -> glm::Mat4 {
        let mat = one();
        let mat = glm::rotate_y(&mat, yaw);
        let mat = glm::rotate_x(&mat, pitch);
        glm::rotate_z(&mat, roll)
    }

    pub fn model(position: &glm::Vec3,
                 orientation: &glm::Vec3,
                 scale: &glm::Vec3) -> glm::Mat4 {
        let scale_matrix = scaling(scale);
        let rotation_matrix = rotation(orientation.y, orientation.x, orientation.z);
        let translation_matrix = translation(position);

        translation_matrix * scale_matrix * rotation_matrix
    }

    pub fn viewport(x: f32, y: f32,
                    dimensions: glm::Vec2,
                    z_near: f32, z_far: f32) -> glm::Mat4 {

        let column0 = [dimensions.x/2.0,     0.0,                  0.0,                    0.0];
        let column1 = [0.0,                  dimensions.y/2.0,     0.0,                    0.0];
        let column2 = [0.0,                  0.0,                  (z_far - z_near) / 2.0, 0.0];
        let column3 = [x + dimensions.x/2.0, y + dimensions.y/2.0, (z_far + z_near) / 2.0, 1.0];

        glm::Mat4::from([column0, column1, column2, column3])
    }

    pub fn projection(y_fov_degrees: f32,
                      aspect: f32,
                      z_near: f32, z_far: f32) -> glm::Mat4 {
        glm::perspective(aspect,
                         y_fov_degrees.to_radians(),
                         z_near, z_far)
    }

    pub fn view(orientation: &glm::Vec3,
                position: &glm::Vec3) -> glm::Mat4 {
        let rotation_matrix: glm::Mat4 = rotation(orientation.y, orientation.x, orientation.z);
        // TODO: Document http://www.opengl-tutorial.org/beginners-tutorials/tutorial-3-matrices/#translation-matrices
        let translation_matrix = translation(position);
        // apply rotation first, translation second
        let camera_transorm = translation_matrix * rotation_matrix;

        // TODO: Document: View transform must use inverses, because it must undo the camera pos/rot towards origin.
        glm::inverse(&camera_transorm)
    }

    pub fn screen_to_world(camera: &Camera) -> glm::Mat4 {
        let screen_dimensions: glm::Vec2 = glm::vec2(camera.screen_dimensions.x as _ , camera.screen_dimensions.y as _);
        let aspect = screen_dimensions.x / screen_dimensions.y;

        let view_matrix = view(&camera.orientation, &camera.position);
        let projection_matrix = projection(camera.y_fov_degrees, aspect,
                                           camera.z_near, camera.z_far);
        let viewport_matrix = viewport(0.0, 0.0,
                                       screen_dimensions,
                                       camera.z_near, camera.z_far);

        let world_to_screen_matrix = viewport_matrix * projection_matrix * view_matrix;
        glm::inverse(&world_to_screen_matrix)
    }
}