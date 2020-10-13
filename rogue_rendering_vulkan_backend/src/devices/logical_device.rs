use crate::util::result::{Result, VulkanError};

use ash::version::InstanceV1_0;
use ash::vk;
use std::collections::HashSet;
use std::collections::HashMap;
use std::convert::TryFrom;

struct QueueFamilyIndices {
    indices: HashMap<vk::QueueFlags, Vec<u32>>,
}

struct QueueFamilyCreateData(u32, u32, Vec<f32>);

impl QueueFamilyIndices {
    fn get_queue_family_data(family_indices: &Vec<u32>) -> Result<QueueFamilyCreateData> {
        if family_indices.is_empty() {
            return Err(VulkanError::LogicalDeviceCreateError);
        }

        Ok(QueueFamilyCreateData(family_indices[0], 1, vec![1.0_f32]))
    }
}

pub fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, required_queue_families: &HashSet<vk::QueueFlags>) -> ash::Device {
    
    let queue_indices = find_queue_families(instance, physical_device, required_queue_families);

    let mut queue_create_infos = Vec::new();
    for (_, queue_family_indices) in queue_indices.indices.iter() {
        let QueueFamilyCreateData(queue_family_index, queue_count, queue_priorities) = QueueFamilyIndices::get_queue_family_data(queue_family_indices).expect("Failed to get queue family from indices");
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
        ..vk::PhysicalDeviceFeatures::default()
    };

    let device_create_info = vk::DeviceCreateInfo {
        queue_create_info_count: u32::try_from(queue_create_infos.len()).expect("Failed to convert usize to u32"),
        p_queue_create_infos: queue_create_infos.as_ptr(),
        p_enabled_features: &device_features as *const vk::PhysicalDeviceFeatures,
        ..vk::DeviceCreateInfo::default()
    };

    unimplemented!()
}

fn find_queue_families(instance: &ash::Instance, physical_device: vk::PhysicalDevice, required_queue_families: &HashSet<vk::QueueFlags>) -> QueueFamilyIndices {
    let device_queue_families = unsafe { 
        instance.get_physical_device_queue_family_properties(physical_device) 
    };

    let mut indices: HashMap<vk::QueueFlags, Vec<u32>> = HashMap::new();
    let mut queue_family_index = 0u32;
    for queue_family in device_queue_families {
        if queue_family.queue_count > 0 {
            for &required_family_flag in required_queue_families.iter() {
                if queue_family.queue_flags.contains(required_family_flag) {
                    let entry = indices.entry(required_family_flag).or_default();
                    entry.push(queue_family_index);
                }
            }
        }
        queue_family_index += 1;
    }
    QueueFamilyIndices {
        indices,
    }
}