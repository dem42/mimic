use ash::{vk, InstanceError};
use image::ImageError;
use mimic_common::{propagate, result::MimicCommonError};
use std::{ffi::OsString, num::TryFromIntError, str::Utf8Error};
use thiserror::Error;
use tobj::LoadError;

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
    // propagating common errors
    #[error(transparent)]
    MimicCommonError(MimicCommonError),
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
    // render commands
    #[error("No render command was available")]
    RenderCommandNotAvailable,
    // shaders
    #[error("Failed to read shader: {shader_file:?}. Reason: {source:?}")]
    ShaderFileReadFailure {
        source: std::io::Error,
        shader_file: OsString,
    },
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
    AshInstanceError(InstanceError),
    #[error(transparent)]
    ImageError(ImageError),
    #[error(transparent)]
    ObjError(LoadError),
    #[error(transparent)]
    OtherVkResult(vk::Result),
    #[error(transparent)]
    VulkanStringConversionError(Utf8Error),
    #[error(transparent)]
    VulkanUsizeConversionError(TryFromIntError),
    #[error("Failed to create a window")]
    WindowCreateFailure,
    #[error("The platform surface stored in window is incorrect")]
    WindowIncorrectPlatformSurface,
}

propagate!(
    VulkanError,
    AshInstanceError as InstanceError,
    using_panic_feature
);
propagate!(VulkanError, ImageError as ImageError, using_panic_feature);
propagate!(VulkanError, ObjError as LoadError, using_panic_feature);
propagate!(
    VulkanError,
    OtherVkResult as vk::Result,
    using_panic_feature
);
propagate!(
    VulkanError,
    VulkanStringConversionError as Utf8Error,
    using_panic_feature
);
propagate!(
    VulkanError,
    VulkanUsizeConversionError as TryFromIntError,
    using_panic_feature
);
propagate!(
    VulkanError,
    MimicCommonError as MimicCommonError,
    using_panic_feature
);
