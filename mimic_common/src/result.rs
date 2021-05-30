use std::{ffi::OsString, io};

use crate::propagate;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MimicCommonError>;

#[derive(Error, Debug)]
pub enum MimicCommonError {
    #[error("Failed to get base directory from executable")]
    ExecutableBaseDirError,
    #[error(transparent)]
    IOError(io::Error),
    #[error("Resource {0:?} failed to resolve")]
    ResourceFailedToResolve(OsString),
}

propagate!(MimicCommonError, IOError as io::Error, using_panic_feature);