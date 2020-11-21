use crate::util::result::{Result, VulkanError};
use crate::vertex_buffers::Vertex;

use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use std::convert::TryFrom;

pub fn create_device_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    vertex_buffer: vk::Buffer,
) -> Result<vk::DeviceMemory> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    let memory_requirements =
        unsafe { logical_device.get_buffer_memory_requirements(vertex_buffer) };

    let memory_type_index = find_memory_type(
        memory_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        mem_properties,
    )?;

    let memory_allocate_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index,
        ..Default::default()
    };

    let device_memory = unsafe { logical_device.allocate_memory(&memory_allocate_info, None)? };

    unsafe { logical_device.bind_buffer_memory(vertex_buffer, device_memory, 0)? }

    Ok(device_memory)
}

pub unsafe fn fill_vertex_buffer(
    logical_device: &ash::Device,
    vertex_buffer_memory: vk::DeviceMemory,
    data: &[Vertex],
) -> Result<()> {
    let size = vk::DeviceSize::try_from(std::mem::size_of_val(&data))?;

    let data_ptr =
        logical_device.map_memory(vertex_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    let data_ptr = data_ptr as *mut Vertex;
    data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len());

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
