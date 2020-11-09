pub mod shader_module;

use crate::graphics_pipeline::shader_module::create_shader_module;
use crate::util::result::Result;

use ash::version::DeviceV1_0;
use ash::vk;
use std::os::raw::c_char;
use std::path::Path;

struct GraphicsPipeline {
    _test: u32,
}

impl GraphicsPipeline {
    pub fn create(logical_device: &ash::Device) -> Result<Self> {

        let vert_shader = create_shader_module(Path::new("shader/spv/hardcoded_triangle.vert.spv"), logical_device)?;
        let frag_shader = create_shader_module(Path::new("shader/spv/hardcoded_triangle.frag.spv"), logical_device)?;

        let vertex_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader,
            p_name: "main".as_ptr() as *const c_char,
            ..Default::default()
        };

        let fragment_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader,
            p_name: "main".as_ptr() as *const c_char,
            ..Default::default()
        };

        let pipeline_stages = [vertex_pipeline_shader_stage_create_info, fragment_pipeline_shader_stage_create_info];

        unsafe {
            logical_device.destroy_shader_module(vert_shader, None);
            logical_device.destroy_shader_module(frag_shader, None);
        }

        Ok(Self {
            _test: 1,
        })
    }
}