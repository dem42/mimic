use crate::devices::queues::QueueType;
use crate::presentation::swap_chain::SwapChainSupportDetails;

use std::collections::HashSet;
use std::convert::TryFrom;
use std::os::raw::c_char;
use std::ffi::CString;

pub struct DeviceRequirements {
    pub required_queues: HashSet<QueueType>,
    pub required_device_extensions: Vec<&'static str>,
    pub is_swap_chain_adequate_check: fn(&SwapChainSupportDetails) -> bool,
}

impl DeviceRequirements {
    pub fn new(
        required_queues: &[QueueType],
        required_device_extensions: &[&'static str],
        is_swap_chain_adequate_check: fn(&SwapChainSupportDetails) -> bool) -> Self 
    {
        Self {
            required_queues: required_queues.iter().copied().collect(),
            required_device_extensions: required_device_extensions.iter().copied().collect(),
            is_swap_chain_adequate_check,
        }
    }

    pub fn get_enabled_extension_count(&self) -> u32 {
        u32::try_from(self.required_device_extensions.len()).unwrap_or(0)
    }

    pub fn get_enabled_extension_names(&self) -> Vec<CString> {
        let mut extension_cstrings = Vec::new();
        for &device_extension in self.required_device_extensions.iter() {
            let cstr = CString::new(device_extension).expect("CString creation for device extension failed");
            extension_cstrings.push(cstr);
        }
        extension_cstrings
    }

    pub fn convert_enabled_extension_names(extensions_cstring: &Vec<CString>) -> Vec<*const c_char> {
        extensions_cstring.iter().map(|x| x.as_ptr()).collect()
    }
}