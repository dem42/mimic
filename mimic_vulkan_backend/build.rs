use mimic_build_utils::{resource_bundle::ResourceBundle, shader_compilation::ShaderCompileParams};
use std::{env, path::{Path, PathBuf}};

fn main() {
    println!("Hello from build.rs! Now from demo");
    
    let vulkan_backend_resource_bundle = ResourceBundle {
        resource_dir_path: PathBuf::from("./res"),
    };

    let shader_compile_params = ShaderCompileParams::new(&vulkan_backend_resource_bundle)
        .expect("Failed to create shader params");
    let shader_srcs = shader_compile_params
        .collect_shader_srcs()
        .expect("Failed to collect shaders srcs");
    println!("Number of shader srcs: {}", shader_srcs.len());

    for shader_src in shader_srcs {
        if let Err(error) = shader_src.compile(&shader_compile_params) {
            println!("cargo:warning={}", error);
        }
    }

    let output_dir = env::var_os("OUT_DIR").unwrap();
    let mut target_dir = Path::new(&output_dir).join("res");    
    vulkan_backend_resource_bundle.copy_bundle_to_location(&mut target_dir).expect("Failed to copy bundle");
}
