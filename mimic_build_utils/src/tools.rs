use std::{env, path::{Path, PathBuf}};

use crate::{build_hacks::get_target_from_out_dir, resource_bundle::ResourceBundle, shader_compilation::ShaderCompileParams};

pub fn process_mimic_app_resources(resource_folder_name: &str) {
    println!("Processing mimic app resources under \"{}\"", resource_folder_name);

    // manifest dir is where the Cargo.toml is for this crate. We can use it know the directory of source files and resources
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let resource_bundle = ResourceBundle::new(PathBuf::from(manifest_dir).join(resource_folder_name));

    let output_dir = env::var_os("OUT_DIR").unwrap();
    println!("cargo:warning={:?}", output_dir);
    let output_dir = get_target_from_out_dir(Path::new(&output_dir).to_owned()).unwrap();
    let mut target_dir = Path::new(&output_dir).join(resource_folder_name);

    let shader_compile_params =
        ShaderCompileParams::new(&resource_bundle, target_dir.as_path())
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

    resource_bundle
        .copy_bundle_to_location(&mut target_dir)
        .expect("Failed to copy bundle");
}