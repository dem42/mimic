use crate::buffers::buffer::Buffer;
use crate::buffers::memory::MemoryCopyable;

use crate::devices::queues::QueueMap;
use crate::util::result::Result;

use ash::vk;
use std::convert::TryFrom;

type IndexType = u16;

impl MemoryCopyable for [IndexType] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut u16;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

pub struct IndexBuffer {
    pub data: Buffer,
    pub index_count: usize,
}

impl IndexBuffer {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<Self> {
        let indices = Self::get_rectangle_indices();
        let size = vk::DeviceSize::try_from(std::mem::size_of_val(&indices))?;

        let index_buffer = Buffer::create_and_fill(
            instance,
            physical_device,
            logical_device,
            command_pool,
            queues,
            size,
            &indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        )?;

        Ok(Self {
            data: index_buffer,
            index_count: indices.len(),
        })
    }

    fn get_rectangle_indices() -> [IndexType; 6] {
        [0, 1, 2, 2, 3, 0]
    }
}
