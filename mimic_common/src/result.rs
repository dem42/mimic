use std::io;

use crate::propagate;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, MimicCommonError>;

#[derive(Error, Debug)]
pub enum MimicCommonError {
    #[error("Failed to get base directory from executable")]
    ExecutableBaseDirError,
    #[error(transparent)]
    IOError(io::Error),
}

propagate!(MimicCommonError, IOError as io::Error, using_panic_feature);
