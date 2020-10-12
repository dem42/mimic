use ash::version::InstanceV1_0;
use ash::vk;
use std::collections::HashSet;
use std::collections::HashMap;

pub struct QueueFamilyIndices {
    indices: HashMap<vk::QueueFlags, Vec<u32>>,
}

pub fn create_logical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, required_queue_families: &HashSet<vk::QueueFlags>) -> ash::Device {
    
    let queue_indices = find_queue_families(instance, physical_device, required_queue_families);

    let queue_priority = 1.0f32;
    let vk_queue_create_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        queue_family_index: queue_indices.get_queue_family_index(),
        queue_count: queue_indices.get_queue_count(),
        p_queue_priorities: &queue_priority,
        ..vk::DeviceQueueCreateInfo::default()
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