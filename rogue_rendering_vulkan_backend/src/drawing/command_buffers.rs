use crate::buffers::index_buffer::IndexBuffer;
use crate::buffers::vertex_buffer::VertexBuffer;
use crate::devices::queues::{QueueFamilyIndices, QueueType};
use crate::graphics_pipeline::GraphicsPipeline;
use crate::presentation::swap_chain::SwapChainContainer;
use crate::uniforms::descriptors::DescriptorData;
use crate::util::result::{Result, VulkanError};

use ash::version::DeviceV1_0;
use ash::vk;

use std::convert::TryFrom;
use std::ptr;

pub fn create_command_pool(
    logical_device: &ash::Device,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<vk::CommandPool> {
    let pool_info = vk::CommandPoolCreateInfo {
        queue_family_index: queue_family_indices.queue_index_map
            [&QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS)],
        ..Default::default()
    };

    let command_pool = unsafe { logical_device.create_command_pool(&pool_info, None)? };

    Ok(command_pool)
}

pub fn create_command_buffers(
    logical_device: &ash::Device,
    command_pool: &vk::CommandPool,
    framebuffers: &Vec<vk::Framebuffer>,
    graphics_pipeline: &GraphicsPipeline,
    swap_chain_container: &SwapChainContainer,
    vertex_buffer: &VertexBuffer,
    index_buffer: &IndexBuffer,
    descriptor_data: &DescriptorData,
) -> Result<Vec<vk::CommandBuffer>> {
    let num_framebuffers = framebuffers.len();

    let allocate_info = vk::CommandBufferAllocateInfo {
        command_pool: *command_pool,
        command_buffer_count: u32::try_from(num_framebuffers)?,
        level: vk::CommandBufferLevel::PRIMARY,
        ..Default::default()
    };

    let command_buffers = unsafe { logical_device.allocate_command_buffers(&allocate_info)? };

    for i in 0..num_framebuffers {
        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            p_inheritance_info: ptr::null(),
            ..Default::default()
        };

        unsafe {
            logical_device.begin_command_buffer(command_buffers[i], &command_buffer_begin_info)?
        }

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            render_pass: graphics_pipeline.render_pass,
            framebuffer: framebuffers[i],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swap_chain_container.swap_chain_extent,
            },
            clear_value_count: u32::try_from(GraphicsPipeline::CLEAR_COLORS.len())?,
            p_clear_values: GraphicsPipeline::CLEAR_COLORS.as_ptr(),
            ..Default::default()
        };

        unsafe {
            logical_device.cmd_begin_render_pass(
                command_buffers[i],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            logical_device.cmd_bind_pipeline(
                command_buffers[i],
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.pipeline,
            );

            let vertex_buffers = [vertex_buffer.data.buffer];
            let offsets: [vk::DeviceSize; 1] = [0];
            logical_device.cmd_bind_vertex_buffers(
                command_buffers[i],
                0,
                &vertex_buffers,
                &offsets,
            );

            logical_device.cmd_bind_index_buffer(
                command_buffers[i],
                index_buffer.data.buffer,
                0,
                vk::IndexType::UINT16,
            );

            if i >= descriptor_data.descriptor_sets.len() {
                return Err(VulkanError::DescriptorSetNotAvailable(i));
            }
            let descriptors_sets_to_bind = [descriptor_data.descriptor_sets[i]];
            logical_device.cmd_bind_descriptor_sets(
                command_buffers[i],
                vk::PipelineBindPoint::GRAPHICS,
                graphics_pipeline.pipeline_layout,
                0,
                &descriptors_sets_to_bind,
                &[],
            );

            // let vertex_count = u32::try_from(vertex_buffer.vertex_count)?;
            // let instance_count = 1; // no instancing
            // let first_vertex = 0;
            // let first_instance = 0;
            // logical_device.cmd_draw(
            //     command_buffers[i],
            //     vertex_count,
            //     instance_count,
            //     first_vertex,
            //     first_instance,
            // );

            let index_count = u32::try_from(index_buffer.index_count)?;
            let instance_count = 1; // no instancing
            let first_index = 0;
            let vertex_offset = 0;
            let first_instance = 0;
            logical_device.cmd_draw_indexed(
                command_buffers[i],
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );

            logical_device.cmd_end_render_pass(command_buffers[i]);

            logical_device.end_command_buffer(command_buffers[i])?;
        }
    }

    Ok(command_buffers)
}
