use ash::vk;
use std::fmt;
use std::num::TryFromIntError;
use std::str::Utf8Error;

pub type Result<T> = std::result::Result<T, VulkanError>;

#[derive(Debug, Clone)]
pub enum VulkanError {
    LogicalDeviceCreateError,
    NoValidationLayers,
    PhysicalDeviceNoGPU,
    QueueCreationFailed,
    RequiredValidationLayersUnsupported,
    // shaders
    ShaderFileReadFailure(String),
    // swap chain errors
    SwapChainFormatsError,
    SwapExtentFailedToGetCurrentMonitor,
    // general vulkan errors
    VkError(String),
    VulkanStringConversionError,
    VulkanUsizeConversionError,
}

impl fmt::Display for VulkanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VulkanError::LogicalDeviceCreateError => write!(f, "Failed to create logical device"),
            VulkanError::NoValidationLayers => write!(f, "No available layers"),
            VulkanError::PhysicalDeviceNoGPU => write!(
                f,
                "Failed to create physical device. No GPU with supported functions"
            ),
            VulkanError::QueueCreationFailed => write!(f, "Failed to create queue indices"),
            VulkanError::RequiredValidationLayersUnsupported => {
                write!(f, "Not all required validation layers are supported")
            },
            VulkanError::ShaderFileReadFailure(error_string) => {
                write!(f, "Failed to read shader: {}", error_string)
            },
            VulkanError::SwapChainFormatsError => write!(f, "Failed to choose a swap chain format"),
            VulkanError::SwapExtentFailedToGetCurrentMonitor => {
                write!(f, "Failed to choose a swap extent")
            },
            VulkanError::VkError(msg) => write!(f, "Error from vk::Result: {}", msg),
            VulkanError::VulkanStringConversionError => {
                write!(f, "Failed to convert vulkan string to string")
            },
            VulkanError::VulkanUsizeConversionError => write!(f, "Failed to convert usize"),
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

impl From<TryFromIntError> for VulkanError {
    fn from(_other: TryFromIntError) -> VulkanError {
        VulkanError::VulkanUsizeConversionError
    }
}
