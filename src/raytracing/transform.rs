pub mod matrix {
    use glm::{BaseFloat, GenFloat};
    use crate::raytracing::Camera;

    pub fn translation(offset: &glm::Vec3) -> glm::Mat4 {
        glm::ext::translate(&num_traits::one(), *offset)
    }

    pub fn scaling(scale: &glm::Vec3) -> glm::Mat4 {
        glm::ext::scale(&num_traits::one(), *scale)
    }

    /// taken from [glm lib](https://github.com/g-truc/glm/blob/master/glm/gtx/euler_angles.inl)
    pub fn rotation(yaw: f32,
                    pitch: f32,
                    roll: f32) -> glm::Mat4 {
        fn glm_ext_eulerangle_xyz<T>(yaw: &T,
                                     pitch: &T,
                                     roll: &T) -> glm::Matrix4<T>
            where T: BaseFloat + GenFloat<T> {
            let tmp_ch = glm::cos(*yaw);
            let tmp_sh = glm::sin(*yaw);
            let tmp_cp = glm::cos(*pitch);
            let tmp_sp = glm::sin(*pitch);
            let tmp_cb = glm::cos(*roll);
            let tmp_sb = glm::sin(*roll);

            let mut result: glm::Matrix4<T> = num_traits::one();
            result[0][0] = tmp_ch * tmp_cb + tmp_sh * tmp_sp * tmp_sb;
            result[0][1] = tmp_sb * tmp_cp;
            result[0][2] = -tmp_sh * tmp_cb + tmp_ch * tmp_sp * tmp_sb;
            result[0][3] = T::zero();
            result[1][0] = -tmp_ch * tmp_sb + tmp_sh * tmp_sp * tmp_cb;
            result[1][1] = tmp_cb * tmp_cp;
            result[1][2] = tmp_sb * tmp_sh + tmp_ch * tmp_sp * tmp_cb;
            result[1][3] = T::zero();
            result[2][0] = tmp_sh * tmp_cp;
            result[2][1] = -tmp_sp;
            result[2][2] = tmp_ch * tmp_cp;
            result[2][3] = T::zero();
            result[3][0] = T::zero();
            result[3][1] = T::zero();
            result[3][2] = T::zero();
            result[3][3] = T::one();

            result
        }
        glm_ext_eulerangle_xyz(&yaw, &pitch, &roll)
    }

    pub fn model(position: &glm::Vec3,
                 orientation: &glm::Vec3,
                 scale: &glm::Vec3) -> glm::Mat4 {
        let scale_matrix = scaling(scale);
        let rotation_matrix = rotation(orientation.y, orientation.x, orientation.z);
        let translation_matrix = translation(position);

        let model_matrix = translation_matrix * scale_matrix * rotation_matrix;
        model_matrix
    }

    pub fn viewport(x: f32, y: f32,
                    width: f32, height: f32,
                    z_near: f32, z_far: f32) -> glm::Mat4 {

        let column0 = glm::vec4(width/2.0, 0.0, 0.0, 0.0);
        let column1 = glm::vec4(0.0, height/2.0, 0.0, 0.0);
        let column2 = glm::vec4(0.0, 0.0, (z_far - z_near) / 2.0, 0.0);
        let column3 = glm::vec4(x + width/2.0, y + height/2.0, (z_far + z_near) / 2.0, 1.0);

        glm::Mat4 {
            c0: column0,
            c1: column1,
            c2: column2,
            c3: column3
        }
    }

    pub fn projection(y_fov_degrees: f32,
                      aspect: f32,
                      z_near: f32, z_far: f32) -> glm::Mat4 {
        glm::ext::perspective(y_fov_degrees.to_radians(),
                              aspect, z_near, z_far)
    }

    pub fn view(orientation: &glm::Vec3,
                position: &glm::Vec3) -> glm::Mat4 {
        let rotation_matrix: glm::Mat4 = rotation(orientation.y, orientation.x, orientation.z);
        // TODO: Document http://www.opengl-tutorial.org/beginners-tutorials/tutorial-3-matrices/#translation-matrices
        let translation_matrix = translation(&position);
        // apply translation first, rotation second
        let camera_transorm = rotation_matrix * translation_matrix;

        // TODO: Document: View transform must use inverses, because it must undo the camera pos/rot towards origin.
        glm::inverse(&camera_transorm)
    }

    pub fn screen_to_world(camera: &Camera) -> glm::Mat4 {
        let aspect = camera.pixel_width as f32 / camera.pixel_height as f32;

        let view_matrix = view(&camera.orientation, &camera.position);
        let projection_matrix = projection(camera.y_fov_degrees, aspect,
                                           camera.z_near, camera.z_far);
        let viewport_matrix = viewport(0.0, 0.0,
                                       camera.pixel_width as f32, camera.pixel_height as f32,
                                       camera.z_near, camera.z_far);

        let world_to_screen_matrix = viewport_matrix * projection_matrix * view_matrix;
        let screen_to_world_matrix = glm::inverse(&world_to_screen_matrix);
        screen_to_world_matrix
    }
}