use ash::vk;
use image::ImageError;
use std::{num::TryFromIntError, str::Utf8Error};
use tobj::LoadError;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, VulkanError>;

#[derive(Error, Debug)]
pub enum VulkanError {
    #[error("Failed to find command buffer with index: {0}")]
    CommandBufferNotAvailable(usize),
    // depth
    #[error("Failed to find supported format")]
    DepthFailedToFindSupportedFormat,
    // descriptors
    #[error("Incorrect number of descriptors sets: {0}")]
    DescriptorSetNotAvailable(usize),
    #[error("Image layout transition not supported: {0}")]
    ImageLayoutTransitionNotSupported(String),
    #[error("Linear blit not supported. Another way is needed to generate mipmaps")]
    ImageLinearBlittingNotSupported,
    #[error("Failed to create logical device")]
    LogicalDeviceCreateError,
    // memory
    #[error("Failed to find suitable memory type")]
    MemoryFailedToFindType,
    // validation
    #[error("No available layers")]
    NoValidationLayers,
    #[error("Failed to create physical device. No GPU with supported functions")]
    PhysicalDeviceNoGPU,
    // queues
    #[error("Failed to create queue indices")]
    QueueCreationFailed,
    #[error("Failed to find graphics queue")]
    QueueGraphicsNotFound,
    #[error("Failed to find present queue")]
    QueuePresentNotFound,
    // Pipeline
    #[error("Failed to create graphics pipeline")]
    PipelineCreateError,
    // validation
    #[error("Not all required validation layers are supported")]
    RequiredValidationLayersUnsupported,
    // shaders
    #[error("Failed to read shader: {0}")]
    ShaderFileReadFailure(String),
    // swap chain errors
    #[error("Failed to choose a swap chain format")]
    SwapChainFormatsError,
    #[error("Failed to choose a swap extent")]
    SwapExtentFailedToGetCurrentMonitor,
    // uniform buffer errors
    #[error("No uniform buffer for swap chain image with index {0}")]
    UniformBufferNotAvailable(usize),
    // fallback errors
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error(transparent)]
    ObjError(#[from] LoadError),
    #[error(transparent)]
    OtherVkResult(#[from] vk::Result),
    #[error(transparent)]
    VulkanStringConversionError(#[from] Utf8Error),
    #[error(transparent)]
    VulkanUsizeConversionError(#[from] TryFromIntError),
}
