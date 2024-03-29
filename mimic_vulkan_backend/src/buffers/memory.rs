use crate::{
    devices::queues::QueueMap,
    drawing::command_buffers::{begin_single_time_commands, end_single_time_commands},
    util::result::{Result, VulkanError},
};
use ash::{
    vk,
};
use mimic_common::uniforms::{UniformSpec, UniformUpdateInput};
use std::convert::TryFrom;
//////////////////////// Traits ///////////////////////
pub trait MemoryCopyable {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut core::ffi::c_void);
}
//////////////////////// Fns ///////////////////////
pub fn copy_buffer(
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
    size: vk::DeviceSize,
    logical_device: &ash::Device,
    command_pool: vk::CommandPool,
    queues: &QueueMap,
) -> Result<()> {
    let command_buffer = begin_single_time_commands(logical_device, command_pool)?;

    let copy_regions = [vk::BufferCopy {
        size,
        ..Default::default()
    }];

    unsafe {
        logical_device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &copy_regions);
    }

    end_single_time_commands(command_buffer, logical_device, queues, command_pool)
}

pub fn create_device_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
    vertex_buffer: vk::Buffer,
    memory_property_requirements: vk::MemoryPropertyFlags,
) -> Result<vk::DeviceMemory> {
    let memory_size_type_requirements =
        unsafe { logical_device.get_buffer_memory_requirements(vertex_buffer) };

    let memory_type_index = find_memory_type(
        instance,
        physical_device,
        memory_size_type_requirements.memory_type_bits,
        memory_property_requirements,
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

pub unsafe fn fill_buffer<T>(
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

pub fn fill_uniform_buffer(
    frame_data_input: UniformUpdateInput,
    uniform_spec: &Box<dyn UniformSpec>,
    logical_device: &ash::Device,
    vertex_buffer_memory: vk::DeviceMemory,
) -> Result<()> {
    let size = vk::DeviceSize::try_from(uniform_spec.uniform_buffer_size())?;
    unsafe {
        let data_target_ptr = logical_device.map_memory(
            vertex_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        uniform_spec.get_uniform_data(frame_data_input, data_target_ptr);

        // the driver is allowed to not immediately copy the data to the memory buffer
        // or the writes may not be visible yet
        // the two ways to address this are either with the COHERENT bit memory requirement
        // or we have to vk::Flush.. and vk::Invalidate..
        // the writes may be visible to buffer but they are only guaranteed to be visible to GPU on next vk::QueueSubmit
        logical_device.unmap_memory(vertex_buffer_memory);
    }

    Ok(())
}

pub fn find_memory_type(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    type_filter_bitfield: u32,
    required_properties: vk::MemoryPropertyFlags,
) -> Result<u32> {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    for (i, mem_type) in mem_properties.memory_types.iter().enumerate() {
        if type_filter_bitfield & (1 << i) != 0
            && mem_type.property_flags.contains(required_properties)
        {
            return Ok(u32::try_from(i)?);
        }
    }
    Err(VulkanError::MemoryFailedToFindType)
}
