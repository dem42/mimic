use crate::result::{MimicCommonError, Result};
use log::{self, info};
use std::{
    env,
    path::{Path, PathBuf},
};
//////////////////////// Structs ///////////////////////
pub struct MimicConfig {
    resource_base_folder: PathBuf,
}
//////////////////////// Impls ///////////////////////
impl MimicConfig {
    pub fn new() -> Result<Self> {
        let current_exe = env::current_exe()?;
        let mut resource_base = current_exe
            .parent()
            .ok_or(MimicCommonError::ExecutableBaseDirError)?;
        if let Some(dir_name) = resource_base.file_name() {
            if dir_name == "examples" {
                resource_base = resource_base
                    .parent()
                    .ok_or(MimicCommonError::ExecutableBaseDirError)?;
            }
        }
        info!("Mimic config created with {}", resource_base.display());
        Ok(MimicConfig {
            resource_base_folder: resource_base.to_owned(),
        })
    }

    pub fn resolve_resource<P>(&self, resource_file_name: P) -> Result<PathBuf>
    where
        P: AsRef<Path>,
    {
        let resolved = self.resource_base_folder.join(resource_file_name);
        if resolved.exists() {
            Ok(resolved)
        } else {
            Err(MimicCommonError::ResourceFailedToResolve(
                resolved.as_os_str().to_owned(),
            ))
        }
    }
}
