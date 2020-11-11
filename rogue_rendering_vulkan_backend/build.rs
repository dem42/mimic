use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

struct ShaderSource {
    shader_path: PathBuf,
}

impl ShaderSource {
    fn new(shader_path: PathBuf) -> Self {
        Self { shader_path }
    }

    fn compile(&self, params: &ShaderCompileParams) -> Result<(), String> {
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

struct ShaderCompileParams {
    input_dir: PathBuf,
    output_dir: PathBuf,
    out_arg_flag: OsString,
}

impl ShaderCompileParams {
    const SHADERS_PATH: &'static str = "./shaders/src";
    const SHADERS_SPV: &'static str = "./shaders/spv";

    fn new() -> Self {
        // shaders path is where common shaders are stored
        // these should get compiled by this build script
        // and if any new files are added then the build script should rerun
        println!("cargo:rerun-if-changed={}", Self::SHADERS_PATH);

        let out_arg_flag = OsStr::new("-o").to_owned();
        let output_dir = Path::new(Self::SHADERS_SPV).to_owned();
        let input_dir = Path::new(Self::SHADERS_PATH).to_owned();
        Self {
            input_dir,
            output_dir,
            out_arg_flag,
        }
    }

    fn collect_shader_srcs(&self) -> io::Result<Vec<ShaderSource>> {
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

fn main() {
    println!("Hello from build.rs! Now from demo");

    let shader_compile_params = ShaderCompileParams::new();
    let shader_srcs = shader_compile_params
        .collect_shader_srcs()
        .expect("Failed to collect shaders srcs");
    println!("Number of shader srcs: {}", shader_srcs.len());

    for shader_src in shader_srcs {
        if let Err(error) = shader_src.compile(&shader_compile_params) {
            println!("cargo:warning={}", error);
        }
    }
}
