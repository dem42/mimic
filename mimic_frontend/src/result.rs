use mimic_vulkan_backend::util::result::VulkanError;
use thiserror::Error;
use winit::error::OsError;

pub type Result<T> = std::result::Result<T, MimicError>;

#[derive(Error, Debug)]
pub enum MimicError {
    #[error(transparent)]
    VulkanError(#[from] VulkanError),
    #[error(transparent)]
    WinitOSError(#[from] OsError),
}
