use crate::devices::queues::QueueType;
use crate::presentation::swap_chain::SwapChainSupportDetails;

use ash::vk;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::ffi::CString;
use std::os::raw::c_char;

pub struct DeviceRequirements {
    pub required_queues: HashSet<QueueType>,
    pub required_device_extensions: Vec<&'static str>,
    pub is_swap_chain_adequate_check: fn(&SwapChainSupportDetails) -> bool,
    pub supported_features_check: fn(&vk::PhysicalDeviceFeatures) -> bool,
}

impl DeviceRequirements {
    pub fn new(
        required_queues: &[QueueType],
        required_device_extensions: &[&'static str],
        is_swap_chain_adequate_check: fn(&SwapChainSupportDetails) -> bool,
        supported_features_check: fn(&vk::PhysicalDeviceFeatures) -> bool,
    ) -> Self {
        Self {
            required_queues: required_queues.iter().copied().collect(),
            required_device_extensions: required_device_extensions.iter().copied().collect(),
            is_swap_chain_adequate_check,
            supported_features_check,
        }
    }

    pub fn get_enabled_extension_count(&self) -> u32 {
        u32::try_from(self.required_device_extensions.len()).unwrap_or(0)
    }

    pub fn get_enabled_extension_names(&self) -> Vec<CString> {
        let mut extension_cstrings = Vec::new();
        for &device_extension in self.required_device_extensions.iter() {
            let cstr = CString::new(device_extension)
                .expect("CString creation for device extension failed");
            extension_cstrings.push(cstr);
        }
        extension_cstrings
    }

    pub fn convert_enabled_extension_names(
        extensions_cstring: &Vec<CString>,
    ) -> Vec<*const c_char> {
        extensions_cstring.iter().map(|x| x.as_ptr()).collect()
    }
}
