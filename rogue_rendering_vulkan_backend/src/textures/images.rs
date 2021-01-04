use crate::buffers::buffer::Buffer;
use crate::buffers::memory::{self, MemoryCopyable};
use crate::util::result::Result;

use ash::vk;
use ash::version::DeviceV1_0;
use image::GenericImageView;
use vk::Extent3D;

pub struct TextureImage {
    pub data: vk::Image,
    pub memory: vk::DeviceMemory,
}

impl MemoryCopyable for [u8] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut u8;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

pub fn create_texture_image(
    texture_name: &str,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    logical_device: &ash::Device,
) -> Result<TextureImage> {
    let image = image::open(texture_name)?;

    let (width, height) = image.dimensions();
    let image_size = (width * height * 4) as vk::DeviceSize;

    let rgba_image = image.into_rgba8();
    let pixels: &Vec<u8> = rgba_image.as_raw();

    let staging_buffer = Buffer::new(
        instance,
        physical_device,
        logical_device,
        image_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;

    unsafe {
        memory::fill_buffer(logical_device, staging_buffer.memory, pixels)?;
    }

    let image_create_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        extent: Extent3D::builder()
            .height(height)
            .width(width)
            .depth(1)
            .build(),
        mip_levels: 1,
        array_layers: 1,
        format: vk::Format::R8G8B8_SRGB,
        tiling: vk::ImageTiling::OPTIMAL,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage: vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let vulkan_image = unsafe {
        logical_device.create_image(&image_create_info, None)?
    };

    let memory_requirements = unsafe {
        logical_device.get_image_memory_requirements(vulkan_image)
    };

    let memory_type_index = memory::find_memory_type(
        instance,
        physical_device,
        memory_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL 
    )?;

    let memory_alloc_info = vk::MemoryAllocateInfo {
        allocation_size: memory_requirements.size,
        memory_type_index,
        ..Default::default()
    };

    let image_device_memory = unsafe {
        let image_device_memory = logical_device.allocate_memory(&memory_alloc_info, None)?;
        logical_device.bind_image_memory(vulkan_image, image_device_memory, 0)?;
        image_device_memory
    };

    Ok(TextureImage {
        data: vulkan_image,
        memory: image_device_memory
    })
}
