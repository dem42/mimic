use crate::buffers::buffer::Buffer;
use crate::buffers::memory::MemoryCopyable;
use crate::presentation::swap_chain::SwapChainContainer;
use crate::util::result::Result;

use ash::vk;
use std::convert::TryFrom;

#[repr(C, align(16))]
pub struct ForceAlignWrapper {
    pub foo: glm::Vec2,
}

// vulkan has very precise memory layout requirements
// specifically mat4 needs to be 16 byte aligned
// but since vec2 is only 8 bytes our model mat is not properly aligned unless we force alignment
#[repr(C, align(16))]
pub struct UniformBufferObject {
    pub force_align_wrapper: ForceAlignWrapper,
    pub model: glm::Mat4,
    pub view: glm::Mat4,
    pub proj: glm::Mat4,
}

impl MemoryCopyable for [UniformBufferObject] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut UniformBufferObject;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

pub fn create_uniform_buffers(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    swap_chain_container: &SwapChainContainer,
) -> Result<Vec<Buffer>> {
    let size = vk::DeviceSize::try_from(std::mem::size_of::<UniformBufferObject>())?;

    let mut uniform_buffers = Vec::new();

    for _i in 0..swap_chain_container.swap_chain_images.len() {
        let new_uniform_buf = Buffer::new(
            instance,
            physical_device,
            logical_device,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        uniform_buffers.push(new_uniform_buf);
    }

    Ok(uniform_buffers)
}
