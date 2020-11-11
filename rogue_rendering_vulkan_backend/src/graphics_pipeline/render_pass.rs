use crate::presentation::swap_chain::SwapChainContainer;
use crate::util::result::Result;

use ash::version::DeviceV1_0;
use ash::vk;

pub fn create_render_pass(
    logical_device: &ash::Device,
    swap_chain_container: &SwapChainContainer,
) -> Result<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription {
        format: swap_chain_container.swap_chain_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    // define a reference into the AttachmentDescription array
    // use the references in render subpasses
    // the attachment is an index into the array passed to p_attachments of renderPassCreateInfo
    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    // note that if p_color_attachemnts is an array
    // then the index of the attachment in this array is referenced directly in
    // layout(location = 0) out vec4 color
    // in fragment shaders
    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        ..Default::default()
    };

    let render_pass_create_info = vk::RenderPassCreateInfo {
        attachment_count: 1,
        p_attachments: &color_attachment,
        subpass_count: 1,
        p_subpasses: &subpass,
        ..Default::default()
    };

    let render_pass = unsafe { logical_device.create_render_pass(&render_pass_create_info, None)? };

    Ok(render_pass)
}
