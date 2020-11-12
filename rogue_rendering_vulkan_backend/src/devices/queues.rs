use crate::devices::requirements::DeviceRequirements;
use crate::util::platform::SurfaceContainer;
use crate::util::result::{Result, VulkanError};

use rustylog::{log, Log};

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::collections::{HashMap, HashSet};

#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub enum QueueType {
    QueueWithFlag(vk::QueueFlags),
    PresentQueue,
}

pub struct QueueFamilyIndices {
    pub indices: HashMap<u32, Vec<QueueType>>,
    pub queue_index_map: HashMap<QueueType, u32>,
}

pub struct QueueFamilyCreateData(pub u32, pub u32, pub Vec<f32>);

#[derive(Debug)]
struct QueueData(u32, HashSet<QueueType>);

impl QueueFamilyIndices {
    pub fn find(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceContainer,
        requirements: &DeviceRequirements,
    ) -> Result<Self> {
        let device_queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_data_points = Vec::new();
        let mut queue_family_index = 0u32;
        for queue_family in device_queue_families {
            if queue_family.queue_count > 0 {
                let mut queue_data = QueueData(queue_family_index, HashSet::new());
                for &required_family in requirements.required_queues.iter() {
                    match required_family {
                        QueueType::QueueWithFlag(queue_flag) => {
                            if queue_family.queue_flags.contains(queue_flag) {
                                queue_data.1.insert(required_family);
                            }
                        }
                        QueueType::PresentQueue => {
                            if has_present_function(surface, physical_device, queue_family_index)? {
                                queue_data.1.insert(required_family);
                            }
                        }
                    }
                }
                if !queue_data.1.is_empty() {
                    queue_data_points.push(queue_data);
                }
            }
            queue_family_index += 1;
        }

        log!(
            Log::Debug,
            "Found queue data points: {:?}",
            queue_data_points
        );

        let mut remaining_queue_types: HashSet<_> =
            requirements.required_queues.iter().copied().collect();
        let mut indices: HashMap<u32, Vec<QueueType>> = HashMap::new();

        loop {
            let mut iter_failed = true;
            queue_data_points.sort_by(|a, b| a.1.len().cmp(&b.1.len()));

            if !queue_data_points.is_empty() {
                iter_failed = false;
                let best = queue_data_points.pop().unwrap();
                for queue_data_point in queue_data_points.iter_mut() {
                    for queue_type in best.1.iter() {
                        queue_data_point.1.remove(&queue_type);
                    }
                }

                let entry = indices.entry(best.0).or_default();
                for queue_type in best.1 {
                    remaining_queue_types.remove(&queue_type);
                    entry.push(queue_type);
                }

                if remaining_queue_types.is_empty() {
                    break;
                }
            }

            if iter_failed {
                return Err(VulkanError::QueueCreationFailed);
            }
        }

        let mut queue_index_map = HashMap::new();
        for (&queue_family_index, queue_types_set) in indices.iter() {
            for &queue_type in queue_types_set {
                queue_index_map.insert(queue_type, queue_family_index);
            }
        }

        log!(Log::Info, "Found indices: {:?}", indices);

        Ok(QueueFamilyIndices {
            indices,
            queue_index_map,
        })
    }

    pub fn get_image_sharing_details(&self) -> (vk::SharingMode, usize, Vec<u32>) {
        let indices_needed: Vec<u32> = self.indices.keys().copied().collect();
        if indices_needed.len() > 1 {
            (
                vk::SharingMode::CONCURRENT,
                indices_needed.len(),
                indices_needed,
            )
        } else {
            (vk::SharingMode::EXCLUSIVE, 0, indices_needed)
        }
    }

    pub fn get_best_queue_family_data(queue_family_index: u32) -> QueueFamilyCreateData {
        QueueFamilyCreateData(queue_family_index, 1, vec![1.0_f32])
    }
}

pub fn create_queues(
    queue_indices: &QueueFamilyIndices,
    logical_device: &ash::Device,
) -> Result<HashMap<QueueType, ash::vk::Queue>> {
    let mut queue_map = HashMap::new();
    for (&queue_type, &queue_family_index) in queue_indices.queue_index_map.iter() {
        let QueueFamilyCreateData(queue_family_index, _, _) =
            QueueFamilyIndices::get_best_queue_family_data(queue_family_index);
        let queue = unsafe { logical_device.get_device_queue(queue_family_index, 0) };
        queue_map.insert(queue_type, queue);
    }
    Ok(queue_map)
}

pub fn has_present_function(
    surface: &SurfaceContainer,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
) -> Result<bool> {
    let is_present_support = unsafe {
        surface.surface_loader.get_physical_device_surface_support(
            physical_device,
            queue_family_index,
            surface.surface,
        )
    }?;
    Ok(is_present_support)
}
