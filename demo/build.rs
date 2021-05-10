use mimic_build_utils::shader_compilation::ShaderCompileParams;

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