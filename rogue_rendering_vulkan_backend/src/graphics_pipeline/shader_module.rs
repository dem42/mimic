use crate::util::result::{Result, VulkanError};

use ash::version::DeviceV1_0;
use ash::vk;
use std::path::Path;

pub fn create_shader_module(
    file_name: &Path,
    logical_device: &ash::Device,
) -> Result<vk::ShaderModule> {
    // the SPIR-V bytecode buffer can be freed right after the shader module has been created
    let code = read_shader_file(file_name)?;

    let shader_module_create_info = vk::ShaderModuleCreateInfo {
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
        ..Default::default()
    };

    let shader_module =
        unsafe { logical_device.create_shader_module(&shader_module_create_info, None)? };

    Ok(shader_module)
}

fn read_shader_file(file_name: &Path) -> Result<Vec<u8>> {
    use std::fs::File;
    use std::io::Read;

    let file_result = File::open(file_name);
    let file = match file_result {
        Err(error) => return Err(VulkanError::ShaderFileReadFailure(format!("{}", error))),
        Ok(file) => file,
    };

    let bytes: Vec<u8> = file.bytes().filter_map(|x| x.ok()).collect();

    Ok(bytes)
}
