use crate::{
    depth::helpers::find_depth_format,
    devices::queues::QueueMap,
    presentation::swap_chain::SwapChainContainer,
    textures::images::{Image, MipmapParam},
    util::result::Result,
};
use ash::{version::DeviceV1_0, vk};

#[derive(Default)]
pub struct DepthResource {
    pub depth_image: Image,
    pub depth_image_view: vk::ImageView,
}

impl DepthResource {
    pub fn new(
        msaa_samples: vk::SampleCountFlags,
        instance: &ash::Instance,
        logical_device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        swap_chain_container: &SwapChainContainer,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<Self> {
        let depth_format = find_depth_format(instance, physical_device)?;
        let mut depth_image = Image::new(
            swap_chain_container.swap_chain_extent.width,
            swap_chain_container.swap_chain_extent.height,
            MipmapParam::NoMipmap,
            msaa_samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            instance,
            physical_device,
            logical_device,
        )?;
        let depth_image_view = depth_image.create_image_view(
            depth_format,
            vk::ImageAspectFlags::DEPTH,
            logical_device,
        )?;

        // explicitly transitioning the depth image buffer's format
        depth_image.transition_image_layout(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            depth_format,
            logical_device,
            command_pool,
            queues,
        )?;

        Ok(DepthResource {
            depth_image,
            depth_image_view,
        })
    }

    pub unsafe fn drop(self, logical_device: &ash::Device) {
        logical_device.destroy_image_view(self.depth_image_view, None);
        logical_device.destroy_image(self.depth_image.image, None);
        logical_device.free_memory(self.depth_image.memory, None);
    }
}
