use ash::vk;
use std::fmt;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, VulkanError>;

#[derive(Debug, Clone)]
pub enum VulkanError {    
    RequiredValidationLayersUnsupported,
    NoValidationLayers,
    VulkanStringConversionError,
    VkError(String),
    LogicalDeviceCreateError,
    PhysicalDeviceNoGPU,
    QueueCreationFailed,
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VulkanError::RequiredValidationLayersUnsupported => write!(f, "Not all required validation layers are supported"),
            VulkanError::NoValidationLayers => write!(f, "No available layers"),
            VulkanError::VulkanStringConversionError => write!(f, "Failed to convert vulkan string to string"),
            VulkanError::VkError(msg) => write!(f, "Error from vk::Result: {}", msg),            
            VulkanError::LogicalDeviceCreateError => write!(f, "Failed to create logical device"),            
            VulkanError::PhysicalDeviceNoGPU => write!(f, "Failed to create physical device. No GPU with supported functions"),            
            VulkanError::QueueCreationFailed => write!(f, "Failed to create queue indices"),            
        }
    }
}

// implement automatic conversion from vk::Result to our Result
// this will be called automatically when we use ?
impl From<vk::Result> for VulkanError {
    fn from(other: vk::Result) -> VulkanError {
        let desc = other.to_string();
        VulkanError::VkError(desc)
    }
}
impl From<Utf8Error> for VulkanError {
    fn from(_other: Utf8Error) -> VulkanError {
        VulkanError::VulkanStringConversionError
    }
}