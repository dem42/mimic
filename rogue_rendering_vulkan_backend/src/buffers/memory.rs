use crate::devices::queues::QueueMap;
use crate::util::result::{Result, VulkanError};

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::convert::TryFrom;

pub trait MemoryCopyable {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void);
}

pub fn copy_buffer(
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
    logical_device: &ash::Device,
    command_pool: vk::CommandPool,
    queues: &QueueMap,
) -> Result<()> {
    let cb_alloc_info = vk::CommandBufferAllocateInfo {
        level: vk::CommandBufferLevel::PRIMARY,
        command_pool,
        command_buffer_count: 1,
        ..Default::default()
    };

    let command_buffer = unsafe { logical_device.allocate_command_buffers(&cb_alloc_info)? };

    if command_buffer.is_empty() {
        // we need to have one command buffer for the copy operation
        return Err(VulkanError::CommandBufferNotAvailable(0));
    }

    // start recording command buffer immediatetly
    let cb_begin_info = vk::CommandBufferBeginInfo {
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        ..Default::default()
    };

    unsafe {
        logical_device.begin_command_buffer(command_buffer[0], &cb_begin_info)?;
    }

    let copy_regions = [vk::BufferCopy {
        size,
        ..Default::default()
    }];

    unsafe {
        logical_device.cmd_copy_buffer(command_buffer[0], src_buffer, dst_buffer, &copy_regions);
        logical_device.end_command_buffer(command_buffer[0])?;
    }

    let submit_info = [vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: command_buffer.as_ptr(),
        ..Default::default()
    }];

    let graphics_queue = queues.get_graphics_queue()?;
    unsafe {
        logical_device.queue_submit(graphics_queue, &submit_info, vk::Fence::null())?;
        logical_device.queue_wait_idle(graphics_queue)?;
        logical_device.free_command_buffers(command_pool, &command_buffer);
    }

    Ok(())
}

pub fn create_device_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    vertex_buffer: vk::Buffer,
    memory_property_requirements: vk::MemoryPropertyFlags,
) -> Result<vk::DeviceMemory> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    let memory_size_type_requirements =
        unsafe { logical_device.get_buffer_memory_requirements(vertex_buffer) };

    let memory_type_index = find_memory_type(
        memory_size_type_requirements.memory_type_bits,
        memory_property_requirements,
        mem_properties,
    )?;

    let memory_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: memory_size_type_requirements.size,
        memory_type_index,
        ..Default::default()
    };

    let device_memory = unsafe { logical_device.allocate_memory(&memory_allocate_info, None)? };

    unsafe { logical_device.bind_buffer_memory(vertex_buffer, device_memory, 0)? }

    Ok(device_memory)
}

pub unsafe fn fill_vertex_buffer<T>(
    logical_device: &ash::Device,
    vertex_buffer_memory: vk::DeviceMemory,
    data: &[T],
) -> Result<()>
where
    [T]: MemoryCopyable,
{
    let size = vk::DeviceSize::try_from(std::mem::size_of_val(&data))?;

    let data_ptr =
        logical_device.map_memory(vertex_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    data.copy_to_mapped_memory(data_ptr);

    // the driver is allowed to not immediately copy the data to the memory buffer
    // or the writes may not be visible yet
    // the two ways to address this are either with the COHERENT bit memory requirement
    // or we have to vk::Flush.. and vk::Invalidate..
    // the writes may be visible to buffer but they are only guaranteed to be visible to GPU on next vk::QueueSubmit
    logical_device.unmap_memory(vertex_buffer_memory);

    Ok(())
}

fn find_memory_type(
    type_filter_bitfield: u32,
    required_properties: vk::MemoryPropertyFlags,
    mem_properties: vk::PhysicalDeviceMemoryProperties,
) -> Result<u32> {
    for (i, mem_type) in mem_properties.memory_types.iter().enumerate() {
        if type_filter_bitfield & (1 << i) != 0
            && mem_type.property_flags.contains(required_properties)
        {
            return Ok(u32::try_from(i)?);
        }
    }
    Err(VulkanError::MemoryFailedToFindType)
}
