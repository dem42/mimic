use crate::{graphics_pipeline::GraphicsPipeline, msaa::multisampling::ColorResource, presentation::{image_views::ImageViews, swap_chain::SwapChainContainer}, util::result::Result};
use ash::{version::DeviceV1_0, vk};
use std::convert::TryFrom;

pub fn create_framebuffers(
    logical_device: &ash::Device,
    graphics_pipeline: &GraphicsPipeline,
    image_views: &ImageViews,
    depth_image_view: vk::ImageView,
    color_resource: &ColorResource,
    swap_chain_container: &SwapChainContainer,
) -> Result<Vec<vk::Framebuffer>> {
    let mut framebuffers = Vec::with_capacity(image_views.image_views.len());
    for image_view in image_views.image_views.iter() {
        // the color attachment is different for every swap chain image
        // but the depth attachment can be the same since we only have one subpass running at a time (due to semaphores)
        // and only the subpass reads/writes to the depth attachment
        let attachments = [color_resource.view, depth_image_view, *image_view];
        let attachment_count = u32::try_from(attachments.len())?;

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            render_pass: graphics_pipeline.render_pass,
            attachment_count,
            p_attachments: attachments.as_ptr(),
            width: swap_chain_container.swap_chain_extent.width,
            height: swap_chain_container.swap_chain_extent.height,
            layers: 1,
            ..Default::default()
        };

        let framebuffer =
            unsafe { logical_device.create_framebuffer(&framebuffer_create_info, None)? };
        framebuffers.push(framebuffer);
    }
    Ok(framebuffers)
}
