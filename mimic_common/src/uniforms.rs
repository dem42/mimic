use log::info;
use nalgebra_glm::{self as glm, Mat4, Vec2, Vec3};

use crate::apptime::AppTime;

//////////////////////// Structs ///////////////////////
/// This struct contains information related to a uniform that we want to use in our shaders
pub struct UniformMetadata {
    pub uniform_buffer_size: usize,
    pub uniform_data_getter: fn(input: UniformUpdateInput<'_>, *mut core::ffi::c_void),
}

#[repr(C, align(16))]
pub struct ForceAlignWrapper {
    pub foo: Vec2,
}

// vulkan has very precise memory layout requirements
// specifically mat4 needs to be 16 byte aligned
// but since vec2 is only 8 bytes our model mat is not properly aligned unless we force alignment
#[repr(C, align(16))]
pub struct UniformBufferObject {
    pub force_align_wrapper: ForceAlignWrapper,
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
}

pub struct UniformUpdateInput<'a> {
    pub swapchain_image_width: u32,
    pub swapchain_image_height: u32,
    pub apptime: &'a AppTime,
}

//////////////////////// Impls ///////////////////////
impl UniformMetadata {
    pub fn new<T>(
        uniform_getter: fn(input: UniformUpdateInput, data_target_ptr: *mut core::ffi::c_void),
    ) -> Self
    where
        T: Sized,
    {
        let size_of_memory_buffer_type = std::mem::size_of::<T>();
        Self {
            uniform_buffer_size: size_of_memory_buffer_type,
            uniform_data_getter: uniform_getter,
        }
    }

    pub fn copy_uniform_to_memory<T>(src: &T, memory_data_target_ptr: *mut core::ffi::c_void)
    where
        T: Sized,
    {
        unsafe {
            let target_ptr = memory_data_target_ptr as *mut T;
            target_ptr.copy_from_nonoverlapping(src as *const T, 1);
        }
    }
}
//////////////////////// Fns ///////////////////////
/// Refreshes the uniform buffer with new data that we want to pass into shaders.
/// The purpose of uniform buffers is to contain data that shaders read. This may be things like transformation matrices needed for 3D rendering.
pub fn update_uniform_buffer(input: UniformUpdateInput, data_target_ptr: *mut core::ffi::c_void) {
    let angle_rad = 0.0; //apptime.elapsed.as_secs_f32() * std::f32::consts::PI / 2.0;
                         // our models for some reason are rotated such that up is z instead of y
    let up_vector = Vec3::new(0., 0., 1.);
    let model = glm::rotate(&Mat4::identity(), angle_rad, &up_vector);

    let view = glm::look_at(
        &glm::Vec3::new(2., 2., 2.),
        &glm::Vec3::new(0., 0., 0.),
        &up_vector,
    );

    let aspect_ratio = input.swapchain_image_width as f32 / input.swapchain_image_height as f32;

    // applying some corrections here because this calculation is for opengl
    // and we have vulkan where in ndc coords the y axis points down
    // also it doesn't use reverse depth
    let mut proj = glm::perspective_fov_rh_zo(
        45.0 * std::f32::consts::PI / 180.0,
        input.swapchain_image_width as f32,
        input.swapchain_image_height as f32,
        0.1,
        10.0,
    );

    if input.apptime.frame % 1000 == 0 {
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

    // the vulkan NDC plane is Y-axis pointing down
    // glm::perspective gives us the opengl computation which has Y-axis pointing up
    // so we need to change the scale of the y axis
    proj.m22 *= -1.0;

    let ubo = UniformBufferObject {
        force_align_wrapper: ForceAlignWrapper {
            foo: glm::Vec2::new(0., 0.),
        },
        model,
        view,
        proj,
    };

    UniformMetadata::copy_uniform_to_memory(&ubo, data_target_ptr);
}
