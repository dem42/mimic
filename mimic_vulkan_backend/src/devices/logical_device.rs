use crate::devices::queues::{QueueFamilyCreateData, QueueFamilyIndices};
use crate::devices::requirements::DeviceRequirements;
use crate::util::result::{Result, VulkanError};
use crate::util::validation::VulkanValidation;

use ash::version::InstanceV1_0;
use ash::vk;
use std::convert::TryFrom;
//////////////////////// Fns ///////////////////////
pub fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_indices: &QueueFamilyIndices,
    requirements: &DeviceRequirements,
    validation: &VulkanValidation,
) -> Result<ash::Device> {
    let mut queue_create_infos = Vec::new();
    for (&queue_family_index, _) in &queue_indices.indices {
        let QueueFamilyCreateData(queue_family_index, queue_count, queue_priorities) =
            QueueFamilyIndices::get_best_queue_family_data(queue_family_index);
        let queue_create_info = vk::DeviceQueueCreateInfo {
            s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
            queue_family_index: queue_family_index,
            queue_count: queue_count,
            p_queue_priorities: queue_priorities.as_ptr(),
            ..vk::DeviceQueueCreateInfo::default()
        };
        queue_create_infos.push(queue_create_info);
    }

    let device_features = vk::PhysicalDeviceFeatures {
        sampler_anisotropy: vk::TRUE,
        ..vk::PhysicalDeviceFeatures::default()
    };

    let enabled_extensions = requirements.get_enabled_extension_names();
    let enabled_extension_cstrs =
        DeviceRequirements::convert_enabled_extension_names(&enabled_extensions);

    let device_create_info = vk::DeviceCreateInfo {
        queue_create_info_count: match u32::try_from(queue_create_infos.len()) {
            Ok(count) => count,
            Err(_) => return Err(VulkanError::LogicalDeviceCreateError),
        },
        p_queue_create_infos: queue_create_infos.as_ptr(),
        p_enabled_features: &device_features as *const vk::PhysicalDeviceFeatures,
        enabled_layer_count: validation.get_enabled_layer_count(),
        pp_enabled_layer_names: validation.get_enabled_layer_names(),
        enabled_extension_count: requirements.get_enabled_extension_count(),
        pp_enabled_extension_names: enabled_extension_cstrs.as_ptr(),
        ..vk::DeviceCreateInfo::default()
    };

    let logical_device =
        unsafe { instance.create_device(physical_device, &device_create_info, None)? };

    Ok(logical_device)
}
