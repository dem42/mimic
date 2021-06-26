use mimic_common::{propagate, result::MimicCommonError};
use mimic_vulkan_backend::util::result::VulkanError;
use thiserror::Error;
use winit::error::OsError;

pub type Result<T> = std::result::Result<T, MimicError>;

#[derive(Error, Debug)]
pub enum MimicError {
    #[error(transparent)]
    MimicCommonError(MimicCommonError),
    #[error(transparent)]
    VulkanError(VulkanError),
    #[error(transparent)]
    WinitOSError(OsError),
}

propagate!(
    MimicError,
    MimicCommonError as MimicCommonError,
    using_panic_feature
);
propagate!(MimicError, VulkanError as VulkanError, using_panic_feature);
propagate!(MimicError, WinitOSError as OsError, using_panic_feature);
