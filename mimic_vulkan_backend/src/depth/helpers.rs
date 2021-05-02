use crate::util::result::{Result, VulkanError};

use ash::version::InstanceV1_0;
use ash::vk;

pub fn has_stencil_component(format: vk::Format) -> bool {
    format == vk::Format::D32_SFLOAT_S8_UINT || format == vk::Format::D24_UNORM_S8_UINT
}

pub fn find_depth_format(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    find_supported_format(
        &vec![
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        instance,
        physical_device,
    )
}

fn find_supported_format(
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::Format> {
    for &candidate_format in candidates {
        let props = unsafe {
            instance.get_physical_device_format_properties(physical_device, candidate_format)
        };

        if tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features) {
            return Ok(candidate_format);
        } else if tiling == vk::ImageTiling::OPTIMAL
            && props.optimal_tiling_features.contains(features)
        {
            return Ok(candidate_format);
        }
    }

    Err(VulkanError::DepthFailedToFindSupportedFormat)
}
