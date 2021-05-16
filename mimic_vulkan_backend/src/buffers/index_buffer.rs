use crate::{
    buffers::buffer::Buffer, devices::queues::QueueMap, models::index::IndexType,
    util::result::Result,
};
use ash::{version::DeviceV1_0, vk};
use std::convert::TryFrom;

#[derive(Default)]
pub struct IndexBuffer {
    pub data: Buffer,
    pub index_count: usize,
}

impl IndexBuffer {
    pub fn new(
        indices: &[IndexType],
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<Self> {
        let size = vk::DeviceSize::try_from(std::mem::size_of::<IndexType>() * indices.len())?;

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

    pub unsafe fn cleanup(self, logical_device: &ash::Device) {
        logical_device.destroy_buffer(self.data.buffer, None);
        logical_device.free_memory(self.data.memory, None);
    }

    pub fn get_rectangle_indices() -> [IndexType; 12] {
        [0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4]
    }
}
