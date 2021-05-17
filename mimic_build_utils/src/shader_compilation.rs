use crate::resource_bundle::ResourceBundle;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Represents a GLSL shader source file in the filesystem.
pub struct ShaderSource {
    shader_path: PathBuf,
}

impl ShaderSource {
    /// Create a new shader source file from the provided path.
    pub fn new(shader_path: PathBuf) -> Self {
        Self { shader_path }
    }

    /// Compile the GLSL shader script into a SPIR-V representation.
    /// Additionally, this emits a cargo:rerun-if-changed for the shader path
    /// so that if there are any changes to the files under this folder, then the build.rs script
    /// will execute again
    pub fn compile(&self, params: &ShaderCompileParams) -> Result<(), String> {
        println!(
            "cargo:rerun-if-changed={}",
            self.shader_path
                .to_str()
                .ok_or(String::from("Failed to get shader path"))?
        );

        let file_name = self
            .shader_path
            .file_name()
            .ok_or("Failed to get file name")?;
        let extension = self
            .shader_path
            .extension()
            .ok_or("Failed to get extension")?;
        let extension = Path::new(extension).with_extension("spv");
        let compiled_file_name = params.output_dir.join(file_name).with_extension(extension);
        let args = [
            self.shader_path.as_os_str(),
            &params.out_arg_flag,
            compiled_file_name.as_os_str(),
        ];

        let command_executable = if cfg!(target_os = "windows") {
            "glslc.exe"
        } else {
            "glslc"
        };
        let command_result = Command::new(command_executable).args(&args).output();
        println!(
            "Running {:?} -> got result: {:?}",
            self.shader_path, command_result
        );
        match command_result {
            Err(error) => Err(format!("Error running command: {}", error)),
            Ok(output) => {
                if !output.status.success() {
                    match String::from_utf8(output.stderr) {
                        Ok(stderr_utf8) => Err(format!("Shader compile error: {}", stderr_utf8)),
                        Err(error) => Err(format!(
                            "Shader unknown compile error. Cannot convert to utf8: {:?}",
                            error
                        )),
                    }
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// This struct represents a directory of shaders. It contains both the input directory where GLSL source files are stored
/// as well as the output directory where the compiled SPIR-V shaders are saved.
pub struct ShaderCompileParams {
    input_dir: PathBuf,
    output_dir: PathBuf,
    out_arg_flag: OsString,
}

impl ShaderCompileParams {
    const SHADERS_PATH: &'static str = "./shaders/src";
    const SHADERS_SPV: &'static str = "./shaders/spv";

    /// Creates an instance of shader compile parameters using the default `./shaders/` folder in the crate folder hierarchy.
    pub fn new(resource_bundle: &ResourceBundle) -> io::Result<Self> {
        let resource_dir = resource_bundle.resource_dir_path.as_path();
        let output_dir = resource_dir
            .join(Self::SHADERS_SPV)
            .to_owned()
            .canonicalize()?;
        let input_dir = resource_dir
            .join(Self::SHADERS_PATH)
            .to_owned()
            .canonicalize()?;

        // shaders path is where common shaders are stored
        // these should get compiled by this build script
        // and if any new files are added then the build script should rerun
        println!("cargo:rerun-if-changed={}", input_dir.display());

        let out_arg_flag = OsStr::new("-o").to_owned();
        println!(
            "Compiling shader sources from {} to {}",
            input_dir.display(),
            output_dir.display(),
        );
        Ok(Self {
            input_dir,
            output_dir,
            out_arg_flag,
        })
    }

    /// Locate all vertex and fragment GLSL shaders.
    pub fn collect_shader_srcs(&self) -> io::Result<Vec<ShaderSource>> {
        let mut result = Vec::new();
        if self.input_dir.is_dir() {
            for entry in fs::read_dir(&self.input_dir)? {
                let entry = entry?;
                let path = entry.path();
                println!("Reading path {}", path.to_str().unwrap());
                if path.is_file() {
                    let file_ext = path.extension();
                    if let Some(ext) = file_ext {
                        if ext == "vert" || ext == "frag" {
                            result.push(ShaderSource::new(path));
                        }
                    }
                }
            }
        }

        Ok(result)
    }
}
