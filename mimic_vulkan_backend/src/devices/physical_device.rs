use crate::devices::queues::{has_present_function, QueueType};
use crate::devices::requirements::DeviceRequirements;
use crate::presentation::swap_chain::SwapChainSupportDetails;
use crate::util::platform::SurfaceContainer;
use crate::util::result::{Result, VulkanError};
use crate::util::tools;

use log::{info, trace};

use ash::vk;
use ash::vk::{version_major, version_minor, version_patch};
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::{BinaryHeap, HashSet};
use std::convert::TryFrom;
use std::f32::consts::E;

#[derive(PartialEq, Eq)]
struct RatedPhyiscalDevice {
    rating: u32,
    physical_device: vk::PhysicalDevice,
    short_description: String,
    long_description: String,
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

pub fn get_physical_device_properties(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
) -> Result<vk::PhysicalDeviceProperties> {
    let physical_device_properties =
        unsafe { instance.get_physical_device_properties(physical_device) };
    Ok(physical_device_properties)
}

// the device is implicitly destroyed when instance is destroyed
pub fn pick_physical_device(
    instance: &ash::Instance,
    surface_container: &SurfaceContainer,
    requirements: &DeviceRequirements,
) -> Result<vk::PhysicalDevice> {
    let physical_devices = unsafe { instance.enumerate_physical_devices()? };
    if physical_devices.is_empty() {
        return Err(VulkanError::PhysicalDeviceNoGpu);
    }

    let mut suitable_physical_devices = BinaryHeap::new();

    for &physical_device in physical_devices.iter() {
        let (rating, short_description, long_description) =
            rate_physical_device(instance, physical_device, surface_container, requirements)?;
        if rating > 0 {
            suitable_physical_devices.push(RatedPhyiscalDevice {
                rating,
                physical_device,
                // TODO: to prevent allocations how could we make these things string only given a certain debug level? cfg flag?
                short_description,
                long_description,
            });
        }
    }

    if let Some(best_suitable_phyiscal_device) = suitable_physical_devices.pop() {
        info!(
            "Best physical device: name = {}, rating = {}",
            best_suitable_phyiscal_device.short_description, best_suitable_phyiscal_device.rating
        );
        if best_suitable_phyiscal_device.rating == 0 {
            info!(
                "Description:\n{}",
                best_suitable_phyiscal_device.long_description
            );
        } else {
            trace!(
                "Description:\n{}",
                best_suitable_phyiscal_device.long_description
            );
        }
        Ok(best_suitable_phyiscal_device.physical_device)
    } else {
        Err(VulkanError::PhysicalDeviceNoGpu)
    }
}

// device is a handle and implements copy
fn rate_physical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    surface_container: &SurfaceContainer,
    requirements: &DeviceRequirements,
) -> Result<(u32, String, String)> {
    let physical_device_properties =
        unsafe { instance.get_physical_device_properties(physical_device) };
    let physical_device_features =
        unsafe { instance.get_physical_device_features(physical_device) };
    let device_queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut rating = 0;
    let mut short_description = String::new();
    let mut long_description = String::new();
    long_description.push_str("Type: ");
    match physical_device_properties.device_type {
        vk::PhysicalDeviceType::CPU => {
            long_description.push_str("Cpu\n");
            rating += 0;
        }
        vk::PhysicalDeviceType::INTEGRATED_GPU => {
            long_description.push_str("Integrated GPU\n");
            rating += 10;
        }
        vk::PhysicalDeviceType::DISCRETE_GPU => {
            long_description.push_str("Discrete GPU\n");
            rating += 1000;
        }
        vk::PhysicalDeviceType::VIRTUAL_GPU => {
            long_description.push_str("Virtual GPU\n");
            rating += 1;
        }
        _ => {
            long_description.push_str("Other\n");
            rating += 0
        }
    };

    let device_name = tools::vk_to_string(&physical_device_properties.device_name);
    if let Ok(device_name) = device_name {
        long_description.push_str(&format!("Name: {}\n", device_name));
        short_description.push_str(&device_name);
    }

    let major_version = version_major(physical_device_properties.api_version);
    let minor_version = version_minor(physical_device_properties.api_version);
    let patch_version = version_patch(physical_device_properties.api_version);
    long_description.push_str(&format!(
        "Version: {}.{}.{}\n",
        major_version, minor_version, patch_version
    ));

    // if we don't match required device extensions then return 0 as rating
    if !(check_device_extensions(
        instance,
        physical_device,
        requirements,
        &mut long_description,
    )?) {
        return Ok((0, short_description, long_description));
    }

    if !((requirements.supported_features_check)(&physical_device_features)) {
        long_description.push_str("Physical device features don't support the necessary features");
        return Ok((0, short_description, long_description));
    }

    // if we have any special requirements as to what needs to be supported we should put it here
    let mut found_queue_families = HashSet::new();
    for (queue_family_idx, queue_family) in device_queue_families.iter().enumerate() {
        long_description.push_str(&format!("Queue Count: {:2} ", queue_family.queue_count));
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            long_description.push_str("| Graphics Queue: supported ");
            found_queue_families.insert(QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS));
        } else {
            long_description.push_str("| Graphics Queue: unsupported ");
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
            long_description.push_str("| Compute Queue: supported ");
            found_queue_families.insert(QueueType::QueueWithFlag(vk::QueueFlags::COMPUTE));
        } else {
            long_description.push_str("| Compute Queue: unsupported ");
        };
        if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
            long_description.push_str("| Transfer Queue: supported ");
            found_queue_families.insert(QueueType::QueueWithFlag(vk::QueueFlags::TRANSFER));
        } else {
            long_description.push_str("| Transfer Queue: unsupported ");
        };
        if queue_family
            .queue_flags
            .contains(vk::QueueFlags::SPARSE_BINDING)
        {
            long_description.push_str("| Sparse binding Queue: supported ");
            found_queue_families.insert(QueueType::QueueWithFlag(vk::QueueFlags::SPARSE_BINDING));
        } else {
            long_description.push_str("| Sparse binding Queue: unsupported ");
        };

        if has_present_function(surface_container, physical_device, queue_family_idx as u32)? {
            long_description.push_str("| Present: supported ");
            found_queue_families.insert(QueueType::PresentQueue);
        } else {
            long_description.push_str("| Present: unsupported ");
        }

        long_description.push('\n');
    }

    if !requirements
        .required_queues
        .is_subset(&found_queue_families)
    {
        return Ok((0, short_description, long_description));
    } else {
        rating += 100
            * u32::try_from(found_queue_families.len()).expect("Failed to convert usize to u32");
    }

    if physical_device_features.geometry_shader == 1 {
        rating += 100;
        long_description.push_str("Geometry Shader supported\n");
    } else {
        long_description.push_str("Geometry Shader unsupported\n");
    };

    let swap_query_support_details =
        SwapChainSupportDetails::query_support(physical_device, &surface_container)?;
    if !((requirements.is_swap_chain_adequate_check)(&swap_query_support_details)) {
        long_description.push_str("Swap chain doesn't pass adequate check");
        return Ok((0, short_description, long_description));
    }

    Ok((rating, short_description, long_description))
}

fn check_device_extensions(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    requirements: &DeviceRequirements,
    description: &mut String,
) -> Result<bool> {
    let available_extensions =
        unsafe { instance.enumerate_device_extension_properties(physical_device)? };

    let mut available_extension_names = vec![];

    let mut required_extensions_found = 0;
    description.push_str("\tAvailable Device Extensions:\n");
    for extension in available_extensions.iter() {
        let extension_name = tools::vk_to_string(&extension.extension_name)?;
        description.push_str(&format!(
            "\t\tName: {}, Version: {}\n",
            extension_name, extension.spec_version
        ));
        if requirements
            .required_device_extensions
            .contains(&&extension_name[..])
        {
            required_extensions_found += 1;
        }
        available_extension_names.push(extension_name);
    }

    let all_required_extensions_found =
        requirements.required_device_extensions.len() == required_extensions_found;
    Ok(all_required_extensions_found)
}
