use crate::util::result::{Result, VulkanError};
use crate::util::tools::vk_to_string;

use log::info;

use ash::version::EntryV1_0;
use std::convert::TryFrom;
use std::os::raw::c_char;
use std::ptr;

pub enum ValidationOptions {
    None,
    Verbose,
}

pub struct VulkanValidation {
    pub is_enabled: bool,
    validation_layer_names: [&'static str; 1],
    options: ValidationOptions,
}

impl VulkanValidation {
    pub const fn enabled(options: ValidationOptions) -> Self {
        VulkanValidation {
            is_enabled: true,
            validation_layer_names: ["VK_LAYER_KHRONOS_validation"],
            options,
        }
    }

    pub const fn disabled() -> Self {
        VulkanValidation {
            is_enabled: false,
            validation_layer_names: [""],
            options: ValidationOptions::None,
        }
    }

    pub fn get_enabled_layer_count(&self) -> u32 {
        if self.is_enabled {
            u32::try_from(self.validation_layer_names.len()).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn get_enabled_layer_names(&self) -> *const *const c_char {
        if self.is_enabled {
            self.validation_layer_names.as_ptr() as *const *const c_char
        } else {
            ptr::null()
        }
    }

    pub fn check_validation_layer_support(&self, entry: &ash::Entry) -> Result<bool> {
        if !self.is_enabled {
            return Ok(true);
        }

        let layer_properties = entry.enumerate_instance_layer_properties()?;

        if layer_properties.is_empty() {
            return Err(VulkanError::NoValidationLayers);
        } else if let ValidationOptions::Verbose = self.options {
            info!("Available layers:");
            for layer in &layer_properties {
                info!("{}", vk_to_string(&layer.layer_name)?);
            }
        }

        for required_validation_layer in &self.validation_layer_names {
            let mut layer_found = false;
            for layer in &layer_properties {
                let layer_name = vk_to_string(&layer.layer_name)?;
                if layer_name == *required_validation_layer {
                    layer_found = true;
                    break;
                }
            }
            if !layer_found {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
