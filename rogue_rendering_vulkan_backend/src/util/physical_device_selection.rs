use crate::util::tools;

use ash::version::InstanceV1_0;
use ash::vk;
use ash::vk::{version_major, version_minor, version_patch};
use std::cmp::{Eq, PartialEq, PartialOrd, Ord, Ordering};
use std::collections::{BinaryHeap, HashSet};
use std::convert::TryFrom;

#[derive(PartialEq, Eq)]
struct RatedPhyiscalDevice {
    rating: u32,
    physical_device: vk::PhysicalDevice,
    description: String,
}

impl Ord for RatedPhyiscalDevice {
    fn cmp(&self, other: &RatedPhyiscalDevice) -> Ordering {
         self.rating.cmp(&other.rating)
    }
}

impl PartialOrd for RatedPhyiscalDevice {
    fn partial_cmp(&self, other: &RatedPhyiscalDevice) -> Option<Ordering> {
         Some(self.cmp(other))
    }
}

// the device is implicitly destroyed when instance is destroyed
pub fn pick_physical_device(instance: &ash::Instance, required_queue_families: &HashSet<vk::QueueFlags>) -> vk::PhysicalDevice {
    let physical_devices = unsafe {
        instance.enumerate_physical_devices().expect("Failed to get phyiscal devices")
    };
    if physical_devices.is_empty() {
        panic!("No GPUs with Vulkan support found");
    }

    let mut suitable_physical_devices = BinaryHeap::new();

    for &physical_device in physical_devices.iter() {
        let (rating, description) = rate_physical_device(instance, physical_device, required_queue_families);
        if rating > 0 {
            suitable_physical_devices.push(RatedPhyiscalDevice {
                rating,
                physical_device,
                description,
            });
        }
    }

    if let Some(best_suitable_phyiscal_device) = suitable_physical_devices.pop() {
        println!("Best physical device: {}", best_suitable_phyiscal_device.description);
        best_suitable_phyiscal_device.physical_device
    } else {
        panic!("Failed to find a suitable GPU!");
    }    
}

// device is a handle and implements copy
fn rate_physical_device(instance: &ash::Instance, physical_device: vk::PhysicalDevice, required_queue_families: &HashSet<vk::QueueFlags>) -> (u32, String) {
    let physical_device_properties = unsafe {
        instance.get_physical_device_properties(physical_device)
    };
    let physical_device_features = unsafe {
        instance.get_physical_device_features(physical_device)
    };
    let device_queue_families = unsafe { 
        instance.get_physical_device_queue_family_properties(physical_device) 
    };

    let mut rating = 0;
    let mut description = String::new();
    description.push_str("Type: ");
    match physical_device_properties.device_type {
        vk::PhysicalDeviceType::CPU => {description.push_str("Cpu\n"); rating += 0;},
        vk::PhysicalDeviceType::INTEGRATED_GPU => {description.push_str("Integrated GPU\n"); rating += 10;},
        vk::PhysicalDeviceType::DISCRETE_GPU => {description.push_str("Discrete GPU\n"); rating += 1000;},
        vk::PhysicalDeviceType::VIRTUAL_GPU => {description.push_str("Virtual GPU\n"); rating += 1;},
        _ => {description.push_str("Other\n"); rating += 0},
    };

    let device_name = tools::vk_to_string(&physical_device_properties.device_name);
    if let Ok(device_name) = device_name {
        description.push_str(&format!("Name: {}\n", device_name));
    }

    let major_version = version_major(physical_device_properties.api_version);
    let minor_version = version_minor(physical_device_properties.api_version);
    let patch_version = version_patch(physical_device_properties.api_version);
    description.push_str(&format!("Version: {}.{}.{}\n", major_version, minor_version, patch_version));

    // if we have any special requirements as to what needs to be supported we should put it here
    let mut found_queue_families = HashSet::new();
    for queue_family in device_queue_families.iter() {
        description.push_str(&format!("Queue Count: {:2} ", queue_family.queue_count));
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            description.push_str("| Graphics Queue: supported ");
            found_queue_families.insert(vk::QueueFlags::GRAPHICS);
        } else {
            description.push_str("| Graphics Queue: unsupported ");
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            description.push_str("| Compute Queue: supported ");
            found_queue_families.insert(vk::QueueFlags::COMPUTE);
        } else {
            description.push_str("| Compute Queue: unsupported ");
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            description.push_str("| Transfer Queue: supported ");
            found_queue_families.insert(vk::QueueFlags::TRANSFER);
        } else {
            description.push_str("| Transfer Queue: unsupported ");
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING) {
            description.push_str("| Sparse binding Queue: supported ");
            found_queue_families.insert(vk::QueueFlags::SPARSE_BINDING);
        } else {
            description.push_str("| Sparse binding Queue: unsupported ");
        };
        description.push_str("\n");
    }
    
    if !required_queue_families.is_subset(&found_queue_families) {
        return (0, description);
    } else {
        rating += 100 * u32::try_from(found_queue_families.len()).expect("Failed to convert usize to u32");
    }

    if physical_device_features.geometry_shader == 1 {
        rating += 100;
        description.push_str("Geometry Shader supported\n");
    } else {
        description.push_str("Geometry Shader unsupported\n");
    };

    (rating, description)
}