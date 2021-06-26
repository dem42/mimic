use std::{
    fs,
    io::Result,
    path::{Path, PathBuf},
};

/// This struct represents a bundle of mimic resource files like textures, models, shaders etc.
/// Bundles need to be moved to a location where the executable can access them using a relative path.
pub struct ResourceBundle {
    pub resource_dir_path: PathBuf,
}

impl ResourceBundle {
    pub fn new(resource_dir_path: PathBuf) -> Self {
        println!("cargo:rerun-if-changed={}", resource_dir_path.display());
        Self { resource_dir_path }
    }

    pub fn copy_bundle_to_location(&self, target_dir: &Path) -> Result<()> {
        let mut accumulated_path = target_dir.to_owned();
        println!(
            "Attempting to create dir {}",
            accumulated_path.as_path().display()
        );
        if !accumulated_path.is_dir() {
            fs::create_dir(accumulated_path.as_path())?;
        }
        Self::copy_recursive(&self.resource_dir_path, &mut accumulated_path)?;
        Ok(())
    }

    fn copy_recursive(source_path: &Path, accumulated_path: &mut PathBuf) -> Result<()> {
        for entry in fs::read_dir(source_path)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(dirname) = path.file_name() {
                accumulated_path.push(dirname);
                if path.is_dir() {
                    if !accumulated_path.is_dir() {
                        fs::create_dir(accumulated_path.as_path())?;
                    }
                    Self::copy_recursive(&path, accumulated_path)?;
                } else {
                    println!(
                        "Attempting to copy {} to {}",
                        path.display(),
                        accumulated_path.as_path().display()
                    );
                    fs::copy(path, accumulated_path.as_path())?;
                }
                accumulated_path.pop();
            }
        }
        Ok(())
    }
}
