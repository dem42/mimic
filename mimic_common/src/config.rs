use crate::result::{MimicCommonError, Result};
use log::{self, info};
use std::{env, path::{Path, PathBuf}};

pub struct MimicConfig {
    resource_base_folder: PathBuf,
}

impl MimicConfig {
    pub fn new() -> Result<Self> {
        let current_exe = env::current_exe()?;
        let resource_base = current_exe.parent().ok_or(MimicCommonError::ExecutableBaseDirError)?;
        info!("Mimic config created with {}", resource_base.display());
        Ok(MimicConfig {
            resource_base_folder: resource_base.to_owned(),
        })
    }

    pub fn resolve_resource<P>(&self, resource_file_name: P) -> PathBuf 
    where
        P: AsRef<Path>
    {
        let resolved = self.resource_base_folder.join(resource_file_name);
        resolved
    }
}