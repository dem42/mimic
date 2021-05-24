use mimic_build_utils::{build_hacks::get_target_from_out_dir, resource_bundle::ResourceBundle, shader_compilation::ShaderCompileParams};
use std::{env, path::{Path, PathBuf}};

fn main() {
    println!("Building crate demo");

    // manifest dir is where the Cargo.toml is for this crate. We can use it know the directory of source files and resources
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let demo_resource_bundle = ResourceBundle {
        resource_dir_path: PathBuf::from(manifest_dir).join("res"),
    };

    let output_dir = env::var_os("OUT_DIR").unwrap();
    println!("cargo:warning={:?}", output_dir);
    let output_dir = get_target_from_out_dir(Path::new(&output_dir).to_owned()).unwrap();
    let mut target_dir = Path::new(&output_dir).join("res");   

    let shader_compile_params =
        ShaderCompileParams::new(&demo_resource_bundle, target_dir.as_path()).expect("Failed to create shader params");
    let shader_srcs = shader_compile_params
        .collect_shader_srcs()
        .expect("Failed to collect shaders srcs");
    println!("Number of shader srcs: {}", shader_srcs.len());

    for shader_src in shader_srcs {
        if let Err(error) = shader_src.compile(&shader_compile_params) {
            println!("cargo:warning={}", error);
        }
    }   
    
    demo_resource_bundle.copy_bundle_to_location(&mut target_dir).expect("Failed to copy bundle");
}
