use crate::buffers::buffer::Buffer;
use crate::buffers::memory::MemoryCopyable;
use crate::presentation::swap_chain::SwapChainContainer;
use crate::util::result::Result;

use ash::vk;
use std::convert::TryFrom;
//////////////////////// Traits ///////////////////////
pub trait UniformBuffer {
    fn update_uniform(&mut self);
}

//////////////////////// Impls ///////////////////////
impl<T: UniformBuffer> MemoryCopyable for [T] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut T;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

//////////////////////// Fns ///////////////////////
pub fn create_uniform_buffers(
    uniform_buffer_size: usize,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    swap_chain_container: &SwapChainContainer,
) -> Result<Vec<Buffer>> {
    let size = vk::DeviceSize::try_from(uniform_buffer_size)?;

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
