use crate::buffers::memory;

use crate::devices::queues::QueueMap;
use crate::util::result::Result;

use ash::vk;
//////////////////////// Structs ///////////////////////
#[derive(Default)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}
//////////////////////// Impls ///////////////////////
impl Buffer {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        memory_property_requirements: vk::MemoryPropertyFlags,
    ) -> Result<Self> {
        let buffer_create_info = vk::BufferCreateInfo {
            size,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let vertex_buffer = unsafe { logical_device.create_buffer(&buffer_create_info, None)? };

        let vertex_buffer_memory = memory::create_device_memory(
            instance,
            physical_device,
            logical_device,
            vertex_buffer,
            memory_property_requirements,
        )?;

        Ok(Self {
            buffer: vertex_buffer,
            memory: vertex_buffer_memory,
        })
    }

    pub fn create_and_fill<T>(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
        buffer_size: vk::DeviceSize,
        data: &[T],
        usage: vk::BufferUsageFlags,
    ) -> Result<Self>
    where
        [T]: memory::MemoryCopyable,
    {
        let staging_buffer = Buffer::new(
            instance,
            physical_device,
            logical_device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            // IMPORTANT: this only works with array slices due to the copy_non_overlapping
            // it does not work with data being a Vec
            memory::fill_buffer(logical_device, staging_buffer.memory, data)?;
        }

        let result_buffer = Buffer::new(
            instance,
            physical_device,
            logical_device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | usage,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        memory::copy_buffer(
            staging_buffer.buffer,
            result_buffer.buffer,
            buffer_size,
            logical_device,
            command_pool,
            queues,
        )?;

        unsafe {
            logical_device.destroy_buffer(staging_buffer.buffer, None);
            logical_device.free_memory(staging_buffer.memory, None);
        }

        Ok(result_buffer)
    }
}
