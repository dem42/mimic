use crate::buffers::buffer::Buffer;
use crate::buffers::memory::{self, MemoryCopyable};
use crate::devices::queues::QueueMap;
use crate::drawing::command_buffers::{begin_single_time_commands, end_single_time_commands};
use crate::util::result::{Result, VulkanError};

use ash::version::DeviceV1_0;
use ash::vk;
use image::GenericImageView;
use vk::Extent3D;

#[derive(Default)]
pub struct Image {
    pub image: vk::Image,
    pub width: u32,
    pub height: u32,
    pub memory: vk::DeviceMemory,
}

impl MemoryCopyable for [u8] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut u8;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

impl Image {
    pub fn new(
        width: u32,
        height: u32,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
    ) -> Result<Self> {
        let image_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            extent: Extent3D::builder()
                .height(height)
                .width(width)
                .depth(1)
                .build(),
            mip_levels: 1,
            array_layers: 1,
            format,
            tiling,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples: vk::SampleCountFlags::TYPE_1,
            ..Default::default()
        };

        let vulkan_image = unsafe { logical_device.create_image(&image_create_info, None)? };

        let memory_requirements =
            unsafe { logical_device.get_image_memory_requirements(vulkan_image) };

        let memory_type_index = memory::find_memory_type(
            instance,
            physical_device,
            memory_requirements.memory_type_bits,
            memory_properties,
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

        Ok(Self {
            image: vulkan_image,
            width,
            height,
            memory: image_device_memory,
        })
    }

    pub fn copy_buffer_to_image(
        &mut self,
        buffer: &Buffer,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<()> {
        let command_buffer = begin_single_time_commands(logical_device, command_pool)?;

        let buffer_image_copy = vk::BufferImageCopy {
            buffer_offset: 0,
            // 0 for row_length and height just says pixels are tightly packed
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_subresource: vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(0)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
            image_offset: vk::Offset3D::builder().x(0).y(0).z(0).build(),
            image_extent: vk::Extent3D::builder()
                .width(self.width)
                .height(self.height)
                .depth(1)
                .build(),
            ..Default::default()
        };

        // specify an array of regions to copy
        let regions = [buffer_image_copy];

        unsafe {
            logical_device.cmd_copy_buffer_to_image(
                command_buffer,
                buffer.buffer,
                self.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &regions,
            );
        }

        end_single_time_commands(command_buffer, logical_device, queues, command_pool)?;

        Ok(())
    }

    pub fn transition_image_layout(
        &mut self,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        _format: vk::Format,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<()> {
        let command_buffer = begin_single_time_commands(logical_device, command_pool)?;

        let (src_access_mask, dst_access_mask, src_stage_mask, dst_stage_mask) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                ) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                _ => {
                    return Err(VulkanError::ImageLayoutTransitionNotSupported(format!(
                        "{:?} -> {:?}",
                        old_layout, new_layout
                    )));
                }
            };

        let image_memory_barrier = vk::ImageMemoryBarrier {
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image: self.image,
            subresource_range: vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
            src_access_mask,
            dst_access_mask,
            ..Default::default()
        };

        unsafe {
            logical_device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                // dependency_flags,
                vk::DependencyFlags::empty(),
                // memory_barriers,
                &[],
                // buffer_memory_barriers,
                &[],
                // image_memory_barriers
                &[image_memory_barrier],
            );
        }

        end_single_time_commands(command_buffer, logical_device, queues, command_pool)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct TextureImage {
    pub name: String,
    pub image: Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl TextureImage {
    pub fn new(
        texture_name: &str,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
        physical_device_properties: &vk::PhysicalDeviceProperties,
    ) -> Result<Self> {
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

        let mut texture_image = Image::new(
            width,
            height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            instance,
            physical_device,
            logical_device,
        )?;

        texture_image.transition_image_layout(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::Format::R8G8B8A8_SRGB,
            logical_device,
            command_pool,
            queues,
        )?;

        texture_image.copy_buffer_to_image(
            &staging_buffer,
            logical_device,
            command_pool,
            queues,
        )?;

        texture_image.transition_image_layout(
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::Format::R8G8B8A8_SRGB,
            logical_device,
            command_pool,
            queues,
        )?;

        unsafe {
            logical_device.destroy_buffer(staging_buffer.buffer, None);
            logical_device.free_memory(staging_buffer.memory, None);
        }

        let view = Self::create_image_view(
            texture_image.image,
            vk::Format::R8G8B8A8_SRGB,
            logical_device,
        )?;

        let sampler = Self::create_texture_sampler(logical_device, physical_device_properties)?;

        Ok(Self {
            name: texture_name.to_owned(),
            image: texture_image,
            view,
            sampler,
        })
    }

    pub unsafe fn drop(self, logical_device: &ash::Device) {
        logical_device.destroy_sampler(self.sampler, None);
        logical_device.destroy_image_view(self.view, None);

        logical_device.destroy_image(self.image.image, None);
        logical_device.free_memory(self.image.memory, None);
    }

    pub fn create_image_view(
        image: vk::Image,
        format: vk::Format,
        logical_device: &ash::Device,
    ) -> Result<vk::ImageView> {
        let image_view_create_info = vk::ImageViewCreateInfo {
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY)
                .build(),
            subresource_range: vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
            ..Default::default()
        };

        let image_view =
            unsafe { logical_device.create_image_view(&image_view_create_info, None)? };

        Ok(image_view)
    }

    pub fn create_texture_sampler(
        logical_device: &ash::Device,
        physical_device_properties: &vk::PhysicalDeviceProperties,
    ) -> Result<vk::Sampler> {
        let sampler_create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::REPEAT,
            address_mode_v: vk::SamplerAddressMode::REPEAT,
            address_mode_w: vk::SamplerAddressMode::REPEAT,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: physical_device_properties.limits.max_sampler_anisotropy,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            max_lod: 0.0,
            min_lod: 0.0,
            mip_lod_bias: 0.0,
            ..Default::default()
        };

        let sampler = unsafe { logical_device.create_sampler(&sampler_create_info, None)? };

        Ok(sampler)
    }
}
