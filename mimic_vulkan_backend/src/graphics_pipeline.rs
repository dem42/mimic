pub mod render_pass;
pub mod shader_module;

use crate::{
    graphics_pipeline::{render_pass::create_render_pass, shader_module::create_shader_module},
    models::vertex::Vertex,
    presentation::swap_chain::SwapChainContainer,
    util::result::{Result, VulkanError},
};
use ash::{vk};
use std::{convert::TryFrom, ffi::CString, path::Path, ptr};
//////////////////////// Structs ///////////////////////
pub struct GraphicsPipeline {
    pub render_pass: vk::RenderPass,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}
//////////////////////// Impls ///////////////////////
impl GraphicsPipeline {
    // order of clear values has to match order of attachments in our render pass
    pub const CLEAR_COLORS: [vk::ClearValue; 2] = [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                // set the initial value in depth buffer to be the furthest value
                // the far plane is 1, the near plane is 0
                depth: 1.0,
                stencil: 0,
            },
        },
    ];

    pub fn new(
        vertex_shader_file: &Path,
        fragment_shader_file: &Path,
        instance: &ash::Instance,
        logical_device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        swap_chain_container: &SwapChainContainer,
        uniform_descriptors: &vk::DescriptorSetLayout,
        msaa_samples: vk::SampleCountFlags,
    ) -> Result<Self> {
        let vert_shader = create_shader_module(vertex_shader_file, logical_device)?;
        let frag_shader = create_shader_module(fragment_shader_file, logical_device)?;

        let main_function = match CString::new("main") {
            Ok(value) => value,
            Err(_) => return Err(VulkanError::PipelineCreateError),
        };
        ////////////////////////////
        // PROGRAMABLE stages
        ////////////////////////////
        let vertex_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::VERTEX,
            module: vert_shader,
            p_name: main_function.as_ptr(),
            ..Default::default()
        };

        let fragment_pipeline_shader_stage_create_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::FRAGMENT,
            module: frag_shader,
            p_name: main_function.as_ptr(),
            ..Default::default()
        };

        let pipeline_stages = [
            vertex_pipeline_shader_stage_create_info,
            fragment_pipeline_shader_stage_create_info,
        ];

        ////////////////////////////
        // FIXED FUNCTION stages
        ////////////////////////////
        let vertex_input_binding = Vertex::get_binding_description()?;
        let vertex_input_attributes = Vertex::get_attribute_descriptions()?;

        let vertex_input_create_info = vk::PipelineVertexInputStateCreateInfo {
            vertex_binding_description_count: 1,
            p_vertex_binding_descriptions: &vertex_input_binding,
            vertex_attribute_description_count: u32::try_from(vertex_input_attributes.len())?,
            p_vertex_attribute_descriptions: vertex_input_attributes.as_ptr(),
            ..Default::default()
        };

        let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo {
            topology: vk::PrimitiveTopology::TRIANGLE_LIST,
            primitive_restart_enable: vk::FALSE,
            ..Default::default()
        };

        // viewport defines the transformation from image to framebuffer
        let viewport = vk::Viewport {
            x: 0.0f32,
            y: 0.0f32,
            width: swap_chain_container.swap_chain_extent.width as f32,
            height: swap_chain_container.swap_chain_extent.height as f32,
            min_depth: 0.0f32,
            max_depth: 1.0f32,
        };

        // scissor defines a filter of which part of the image to show
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swap_chain_container.swap_chain_extent,
        };

        let viewport_create_info = vk::PipelineViewportStateCreateInfo {
            viewport_count: 1,
            p_viewports: &viewport,
            scissor_count: 1,
            p_scissors: &scissor,
            ..Default::default()
        };

        let rasterization_create_info = vk::PipelineRasterizationStateCreateInfo {
            depth_clamp_enable: vk::FALSE,
            rasterizer_discard_enable: vk::FALSE,
            polygon_mode: vk::PolygonMode::FILL,
            line_width: 1.0f32,
            cull_mode: vk::CullModeFlags::BACK,
            // TODO
            // front_face value depends a lot on what model we are trying to show
            front_face: vk::FrontFace::COUNTER_CLOCKWISE,
            depth_bias_enable: vk::FALSE,
            ..Default::default()
        };

        let multisampling_create_info = vk::PipelineMultisampleStateCreateInfo {
            sample_shading_enable: vk::FALSE,
            rasterization_samples: msaa_samples,
            ..Default::default()
        };

        let color_blend_attachment_state = vk::PipelineColorBlendAttachmentState {
            color_write_mask: vk::ColorComponentFlags::RGBA,
            blend_enable: vk::TRUE,
            src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
            dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            color_blend_op: vk::BlendOp::ADD,
            src_alpha_blend_factor: vk::BlendFactor::ONE,
            dst_alpha_blend_factor: vk::BlendFactor::ZERO,
            alpha_blend_op: vk::BlendOp::ADD,
        };

        let color_blending_create_info = vk::PipelineColorBlendStateCreateInfo {
            logic_op_enable: vk::FALSE,
            attachment_count: 1,
            p_attachments: &color_blend_attachment_state,
            ..Default::default()
        };

        // enable depth testing in graphics pipleine
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
            depth_test_enable: vk::TRUE,
            depth_write_enable: vk::TRUE,
            // lower means closer, so keep fragments that have less depth
            depth_compare_op: vk::CompareOp::LESS,
            depth_bounds_test_enable: vk::FALSE,
            min_depth_bounds: 0.0,
            max_depth_bounds: 1.0,
            stencil_test_enable: vk::FALSE,
            ..Default::default()
        };

        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: uniform_descriptors,
            ..Default::default()
        };

        let render_pass = create_render_pass(
            instance,
            logical_device,
            physical_device,
            swap_chain_container,
            msaa_samples,
        )?;

        let pipeline_layout =
            unsafe { logical_device.create_pipeline_layout(&pipeline_layout_create_info, None)? };

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo {
            // programmable stages
            stage_count: u32::try_from(pipeline_stages.len())?,
            p_stages: pipeline_stages.as_ptr(),
            // fixed function stages
            p_vertex_input_state: &vertex_input_create_info,
            p_input_assembly_state: &input_assembly_create_info,
            p_viewport_state: &viewport_create_info,
            p_rasterization_state: &rasterization_create_info,
            p_multisample_state: &multisampling_create_info,
            p_depth_stencil_state: &depth_stencil,
            p_color_blend_state: &color_blending_create_info,
            p_dynamic_state: ptr::null(),
            // layout defining uniforms etc
            layout: pipeline_layout,
            // render pass and index of subpass where pipeline will be used
            render_pass,
            subpass: 0,
            base_pipeline_handle: vk::Pipeline::null(),
            base_pipeline_index: -1,
            ..Default::default()
        };

        let graphics_pipeline_infos = [pipeline_create_info];
        let graphics_pipeline_result = unsafe {
            logical_device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                &graphics_pipeline_infos,
                None,
            )
        };

        let graphics_pipelines = match graphics_pipeline_result {
            Ok(pipelines) => pipelines,
            Err((_, result)) => return Err(VulkanError::from(result)),
        };

        if graphics_pipelines.is_empty() {
            return Err(VulkanError::PipelineCreateError);
        }
        let pipeline = graphics_pipelines[0];

        unsafe {
            logical_device.destroy_shader_module(vert_shader, None);
            logical_device.destroy_shader_module(frag_shader, None);
        }

        Ok(Self {
            render_pass,
            pipeline_layout,
            pipeline,
        })
    }
}
