use crate::presentation::swap_chain::SwapChainContainer;
use crate::util::result::Result;

use ash::vk;
//////////////////////// Structs ///////////////////////
pub struct ImageViews {
    pub image_views: Vec<vk::ImageView>,
}
//////////////////////// Impls ///////////////////////
impl ImageViews {
    pub fn new(
        logical_device: &ash::Device,
        swap_chain_container: &SwapChainContainer,
    ) -> Result<Self> {
        let mut image_views = Vec::with_capacity(swap_chain_container.swap_chain_images.len());

        for &image in &swap_chain_container.swap_chain_images {
            let image_view_create_info = vk::ImageViewCreateInfo {
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: swap_chain_container.swap_chain_format.format,
                components: vk::ComponentMapping::builder()
                    .r(vk::ComponentSwizzle::IDENTITY)
                    .g(vk::ComponentSwizzle::IDENTITY)
                    .b(vk::ComponentSwizzle::IDENTITY)
                    .a(vk::ComponentSwizzle::IDENTITY)
                    .build(),
                subresource_range: vk::ImageSubresourceRange::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
                ..Default::default()
            };

            let image_view =
                unsafe { logical_device.create_image_view(&image_view_create_info, None)? };

            image_views.push(image_view);
        }

        Ok(ImageViews { image_views })
    }
}
