use crate::buffers::buffer::Buffer;
use crate::buffers::memory::MemoryCopyable;

use crate::devices::queues::QueueMap;
use crate::util::result::Result;

use ash::version::DeviceV1_0;
use ash::vk;
use std::convert::TryFrom;

type IndexType = u16;

impl MemoryCopyable for [IndexType] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut u16;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

#[derive(Default)]
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

    pub unsafe fn drop(self, logical_device: &ash::Device) {
        logical_device.destroy_buffer(self.data.buffer, None);
        logical_device.free_memory(self.data.memory, None);
    }

    fn get_rectangle_indices() -> [IndexType; 12] {
        [
            0, 1, 2, 2, 3, 0,
            4, 5, 6, 6, 7, 4,
        ]
    }
}
