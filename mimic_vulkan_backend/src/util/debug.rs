use crate::util::validation::VulkanValidation;
use log::{error, info, trace, warn};

use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

/// the callback function used in Debug Utils.
unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[General]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[Performance]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[Validation]",
        _ => "[Unknown]",
    };
    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            trace!("[Verbose]{}{:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            info!("[Info]{}{:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            warn!("[Warning]{}{:?}", types, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            error!("[Error]{}{:?}", types, message);
        }
        _ => {
            error!("[Unkown]{}{:?}", types, message);
        }
    };

    vk::FALSE
}

pub struct VulkanDebug {
    debug_utils: ash::extensions::ext::DebugUtils,
    debug_messenger: Option<ash::vk::DebugUtilsMessengerEXT>,
}

impl VulkanDebug {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        validation: &VulkanValidation,
    ) -> Self {
        let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);

        if validation.is_enabled {
            let create_info = Self::populate_debug_create_info();

            let debug_messenger = unsafe {
                debug_utils
                    .create_debug_utils_messenger(&create_info, None)
                    .expect("Failed to create debug utils messenger")
            };
            Self {
                debug_utils,
                debug_messenger: Some(debug_messenger),
            }
        } else {
            Self {
                debug_utils,
                debug_messenger: None,
            }
        }
    }

    pub unsafe fn destroy_debug_messenger(&mut self) {
        if let Some(debug_messenger) = self.debug_messenger {
            self.debug_utils
                .destroy_debug_utils_messenger(debug_messenger, None);
        }
    }

    pub fn get_creation_destruction_debug_create_info(
        validation: &VulkanValidation,
    ) -> *const c_void {
        if validation.is_enabled {
            let create_info = Self::populate_debug_create_info();
            (&create_info as *const vk::DebugUtilsMessengerCreateInfoEXT) as *const c_void
        } else {
            ptr::null()
        }
    }

    fn populate_debug_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            p_next: ptr::null(),
            flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_debug_utils_callback),
            p_user_data: ptr::null_mut(),
        }
    }
}
