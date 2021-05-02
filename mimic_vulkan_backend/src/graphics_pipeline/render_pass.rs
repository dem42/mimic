use crate::{depth::helpers, presentation::swap_chain::SwapChainContainer, util::result::Result};

use ash::version::DeviceV1_0;
use ash::vk;
use std::convert::TryFrom;

pub fn create_render_pass(
    instance: &ash::Instance,
    logical_device: &ash::Device,
    physical_device: vk::PhysicalDevice,
    swap_chain_container: &SwapChainContainer,
    msaa_samples: vk::SampleCountFlags,
) -> Result<vk::RenderPass> {
    // setup the descriptions for the attachments used by the render pass
    let color_attachment = vk::AttachmentDescription {
        format: swap_chain_container.swap_chain_format.format,
        samples: msaa_samples,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        // our final layout isn't present because we are using MSAA and so the color attachment
        // cannot be present immediately. first it must be resolved
        final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let depth_attachment = vk::AttachmentDescription {
        format: helpers::find_depth_format(instance, physical_device)?,
        samples: msaa_samples,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let color_attachment_resolve = vk::AttachmentDescription {
        format: swap_chain_container.swap_chain_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::DONT_CARE,
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

    let depth_attachment_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let color_attachment_resolve_ref = vk::AttachmentReference {
        attachment: 2,
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
        p_depth_stencil_attachment: &depth_attachment_ref,
        // pointing to the resolve attachment ref is enough to tell the graphics subpass to do a resolve operation
        p_resolve_attachments: &color_attachment_resolve_ref,
        ..Default::default()
    };

    // subpasses can specify what needs to happen before by using subpass dependencies
    // this is done because image layout transitions which subpasses specify happen automatically
    // but due to synchronization we need to make sure we do them at the right time
    // which is what we use the subpass dependencies for here
    let subpass_dependency = vk::SubpassDependency {
        // special value for the operations that happen before (if in src) or after (in dst) subpasses
        src_subpass: vk::SUBPASS_EXTERNAL,
        // index of our subpass
        dst_subpass: 0,
        // wait for swapchain to finish reading from the image, before we access it
        // likewise wait for early fragments test to do the read from depth image before we access it
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        src_access_mask: vk::AccessFlags::empty(),
        // prevent image layout transition from happening until it is necessary (when we start writing to it)
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
            | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        ..Default::default()
    };

    // the indices of the attachments in this array is what we use as the "attachment" field in the attachement refs
    let attachments = [color_attachment, depth_attachment, color_attachment_resolve];
    let attachment_count = u32::try_from(attachments.len())?;
    let render_pass_create_info = vk::RenderPassCreateInfo {
        attachment_count,
        p_attachments: attachments.as_ptr(),
        subpass_count: 1,
        p_subpasses: &subpass,
        dependency_count: 1,
        p_dependencies: &subpass_dependency,
        ..Default::default()
    };

    let render_pass = unsafe { logical_device.create_render_pass(&render_pass_create_info, None)? };

    Ok(render_pass)
}
