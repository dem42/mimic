use crate::graphics_pipeline::GraphicsPipeline;
use crate::presentation::image_views::ImageViews;
use crate::presentation::swap_chain::SwapChainContainer;
use crate::util::result::Result;

use ash::version::DeviceV1_0;
use ash::vk;

use std::convert::TryFrom;

pub fn create_framebuffers(
    logical_device: &ash::Device,
    graphics_pipeline: &GraphicsPipeline,
    image_views: &ImageViews,
    swap_chain_container: &SwapChainContainer,
) -> Result<Vec<vk::Framebuffer>> {
    let mut framebuffers = Vec::with_capacity(image_views.image_views.len());
    for image_view in image_views.image_views.iter() {
        let attachments = [*image_view];

        let framebuffer_create_info = vk::FramebufferCreateInfo {
            render_pass: graphics_pipeline.render_pass,
            attachment_count: u32::try_from(attachments.len())?,
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
