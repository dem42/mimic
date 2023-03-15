use log::info;
use nalgebra_glm as glm;
//////////////////////// Structs ///////////////////////
#[derive(Clone, Default)]
pub struct Camera {
    pub position: glm::Vec3,
    pub forward: glm::Vec3,
    pub up: glm::Vec3,
}
//////////////////////// Impls ///////////////////////
impl Camera {
    pub fn new(position: glm::Vec3, forward: glm::Vec3, up: glm::Vec3) -> Self {
        Self {
            position,
            forward,
            up,
        }
    }

    /// Compare the computed projection matrix to the values from a manual calculation
    pub fn debug_print_projection_matrix(&self, image_width: f32, image_height: f32) {
        let aspect_ratio = image_width / image_height;
        let proj = self.get_projection_matrix(image_width, image_height);

        let focal_length = 1.0 / ((45.0 * std::f32::consts::PI / 180.0) / 2.0).tan();
        let a = 10.0 / (0.1 - 10.0);
        let b = (0.1 * 10.0) / (0.1 - 10.0);
        info!(
            "{}, {}, {}, {}",
            focal_length / aspect_ratio,
            -focal_length,
            a,
            b
        );
        info!(
            "Proj:\n[{}, {}, {}, {}]\n[{}, {}, {}, {}]\n[{}, {}, {}, {}]\n[{}, {}, {}, {}]",
            proj.m11,
            proj.m12,
            proj.m13,
            proj.m14,
            proj.m21,
            proj.m22,
            proj.m23,
            proj.m24,
            proj.m31,
            proj.m32,
            proj.m33,
            proj.m34,
            proj.m41,
            proj.m42,
            proj.m43,
            proj.m44,
        );
    }

    /// Get the projection matrix which describes the parameters of projecting the 3D world onto a 2D image
    pub fn get_projection_matrix(&self, image_width: f32, image_height: f32) -> glm::Mat4 {
        // applying some corrections here because this calculation is for opengl
        // and we have vulkan where in ndc coords the y axis points down
        // also it doesn't use reverse depth
        let mut proj = glm::perspective_fov_rh_zo(
            45.0 * std::f32::consts::PI / 180.0,
            image_width,
            image_height,
            0.1,
            10.0,
        );
        // the vulkan NDC plane is Y-axis pointing down
        // glm::perspective gives us the opengl computation which has Y-axis pointing up
        // so we need to change the scale of the y axis
        proj.m22 *= -1.0;
        proj
    }

    /// Get the camera's view matrix which describes the position and rotate (direction) of the camera.
    /// Remember that we don't actually move/rotate the camera at any time. Rather we apply the inverse transformation to all models in the world
    pub fn get_view_matrix(&self) -> glm::Mat4 {
        let look_at_point = &self.position + &self.forward;
        let view = glm::look_at(&self.position, &look_at_point, &self.up);
        view
    }
}
