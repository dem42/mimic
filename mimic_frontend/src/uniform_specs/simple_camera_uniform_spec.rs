use mimic_common::uniforms::{UniformBufferObject, UniformSpec, copy_uniform_to_memory};

use crate::cameras::camera::Camera;

//////////////////////// Structs ///////////////////////
pub struct SimpleCameraUniformSpec {
    camera: Camera,
    static_model_transform: glm::Mat4,
}
//////////////////////// Impls ///////////////////////
impl SimpleCameraUniformSpec {
    pub fn new(camera_pitch: f32) -> Self {
        let camera = Camera::new(
            glm::vec3(0., 0., 3.),
            glm::vec3(0., 0., -1.),
            glm::vec3(0., 1., 0.),
        );
        let static_model_transform: glm::Mat4 = glm::rotation(camera_pitch, &glm::Vec3::x_axis());
        Self {
            camera,
            static_model_transform,
        }
    }
}

impl UniformSpec for SimpleCameraUniformSpec {
    fn get_uniform_data(
        &self,
        input: mimic_common::uniforms::UniformUpdateInput<'_>,
        memory_target_ptr: *mut core::ffi::c_void,
    ) {
        let width = input.swapchain_image_width as f32;
        let height = input.swapchain_image_height as f32;
        let proj = self.camera.get_projection_matrix(width, height);
        let view = self.camera.get_view_matrix();
        let model = self.static_model_transform * glm::Mat4::identity();

        let ubo = UniformBufferObject {
            model,
            view,
            proj,
        };

        copy_uniform_to_memory(&ubo, memory_target_ptr);
    }

    fn uniform_buffer_size(&self) -> usize {
        std::mem::size_of::<UniformBufferObject>()
    }
}