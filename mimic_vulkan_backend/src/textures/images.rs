use crate::{
    buffers::{
        buffer::Buffer,
        memory::{self, MemoryCopyable},
    },
    depth::helpers,
    devices::queues::QueueMap,
    drawing::command_buffers::{begin_single_time_commands, end_single_time_commands},
    util::result::{Result, VulkanError},
};

use ash::{
    version::{DeviceV1_0, InstanceV1_0},
    vk,
};
use mimic_common::texture::TextureSource;
use std::{cmp::max, convert::TryFrom, f32};
//////////////////////// Enums ///////////////////////
#[derive(Debug)]
pub enum MipmapParam {
    NoMipmap,
    UseRuntimeMipmap,
}
//////////////////////// Structs ///////////////////////
#[derive(Default)]
pub struct Image {
    pub image: vk::Image,
    pub width: u32,
    pub height: u32,
    pub memory: vk::DeviceMemory,
    pub mip_levels: u32,
}

#[derive(Default)]
pub struct TextureImage {
    pub texture_source: Option<Box<dyn TextureSource>>,
    pub image: Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}
//////////////////////// Impls ///////////////////////
impl Image {
    pub fn new(
        width: u32,
        height: u32,
        mipmap_param: MipmapParam,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        memory_properties: vk::MemoryPropertyFlags,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
    ) -> Result<Self> {
        // number of mip level = how many times we can scale the image down by a half
        let mip_levels = match mipmap_param {
            MipmapParam::UseRuntimeMipmap => {
                ((max(width, height) as f32).log2().floor() as u32) + 1
            }
            MipmapParam::NoMipmap => 1,
        };

        let image_create_info = vk::ImageCreateInfo {
            image_type: vk::ImageType::TYPE_2D,
            extent: vk::Extent3D::builder()
                .height(height)
                .width(width)
                .depth(1)
                .build(),
            mip_levels,
            array_layers: 1,
            format,
            tiling,
            initial_layout: vk::ImageLayout::UNDEFINED,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            samples: num_samples,
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
            mip_levels,
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

        end_single_time_commands(command_buffer, logical_device, queues, command_pool)
    }

    pub fn transition_image_layout(
        &mut self,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        format: vk::Format,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<()> {
        let command_buffer = begin_single_time_commands(logical_device, command_pool)?;

        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            if helpers::has_stencil_component(format) {
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
            } else {
                vk::ImageAspectFlags::DEPTH
            }
        } else {
            vk::ImageAspectFlags::COLOR
        };

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
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
                    (
                        vk::AccessFlags::empty(),
                        vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                            | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                        vk::PipelineStageFlags::TOP_OF_PIPE,
                        vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                    )
                }
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
                .aspect_mask(aspect_mask)
                .base_mip_level(0)
                .level_count(self.mip_levels)
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

        end_single_time_commands(command_buffer, logical_device, queues, command_pool)
    }

    pub fn create_image_view(
        &self,
        format: vk::Format,
        aspect_flags: vk::ImageAspectFlags,
        logical_device: &ash::Device,
    ) -> Result<vk::ImageView> {
        let image_view_create_info = vk::ImageViewCreateInfo {
            image: self.image,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            components: vk::ComponentMapping::builder()
                .r(vk::ComponentSwizzle::IDENTITY)
                .g(vk::ComponentSwizzle::IDENTITY)
                .b(vk::ComponentSwizzle::IDENTITY)
                .a(vk::ComponentSwizzle::IDENTITY)
                .build(),
            subresource_range: vk::ImageSubresourceRange::builder()
                .aspect_mask(aspect_flags)
                .base_mip_level(0)
                .level_count(self.mip_levels)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
            ..Default::default()
        };

        let image_view =
            unsafe { logical_device.create_image_view(&image_view_create_info, None)? };

        Ok(image_view)
    }
}

impl MemoryCopyable for [u8] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut u8;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}

impl TextureImage {
    pub fn new(
        texture_source: Box<dyn TextureSource>,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
        physical_device_properties: &vk::PhysicalDeviceProperties,
    ) -> Result<Self> {
        let staging_buffer = Buffer::new(
            instance,
            physical_device,
            logical_device,
            texture_source.get_image_size() as vk::DeviceSize,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        unsafe {
            memory::fill_buffer(logical_device, staging_buffer.memory, texture_source.get_pixels())?;
        }

        let mut texture_image = Image::new(
            texture_source.get_width(),
            texture_source.get_height(),
            MipmapParam::UseRuntimeMipmap,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            // the image has the usage
            // 1) transfer source for Blit operations to create mip levels
            // 2) transfer dest for copying staging buffer into it
            // 3) sampled for usage in a sampler in a shader
            vk::ImageUsageFlags::TRANSFER_SRC
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::SAMPLED,
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

        // as part of generating mipmaps all the mip levels transition to the shader read optimal layout
        Self::generate_mipmaps(
            &texture_image,
            vk::Format::R8G8B8A8_SRGB,
            instance,
            logical_device,
            physical_device,
            command_pool,
            queues,
        )?;

        unsafe {
            logical_device.destroy_buffer(staging_buffer.buffer, None);
            logical_device.free_memory(staging_buffer.memory, None);
        }

        let view = texture_image.create_image_view(
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            logical_device,
        )?;

        let sampler = Self::create_texture_sampler(
            logical_device,
            physical_device_properties,
            texture_image.mip_levels,
        )?;

        Ok(Self {
            texture_source: Some(texture_source),
            image: texture_image,
            view,
            sampler,
        })
    }

    pub unsafe fn cleanup(self, logical_device: &ash::Device) {
        logical_device.destroy_sampler(self.sampler, None);
        logical_device.destroy_image_view(self.view, None);

        logical_device.destroy_image(self.image.image, None);
        logical_device.free_memory(self.image.memory, None);
    }

    pub fn create_texture_sampler(
        logical_device: &ash::Device,
        physical_device_properties: &vk::PhysicalDeviceProperties,
        mip_levels: u32,
    ) -> Result<vk::Sampler> {
        let sampler_create_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            anisotropy_enable: vk::TRUE,
            max_anisotropy: physical_device_properties.limits.max_sampler_anisotropy,
            border_color: vk::BorderColor::INT_OPAQUE_BLACK,
            unnormalized_coordinates: vk::FALSE,
            compare_enable: vk::FALSE,
            compare_op: vk::CompareOp::ALWAYS,
            mipmap_mode: vk::SamplerMipmapMode::LINEAR,
            max_lod: mip_levels as f32,
            min_lod: 0.0,
            mip_lod_bias: 0.0,
            ..Default::default()
        };

        let sampler = unsafe { logical_device.create_sampler(&sampler_create_info, None)? };

        Ok(sampler)
    }

    // TODO: that runtime generation of mipmaps is worse than reading them from the file
    fn generate_mipmaps(
        image: &Image,
        format: vk::Format,
        instance: &ash::Instance,
        logical_device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<()> {
        // before doing anything, we need to check if the format supports linear blitting
        let physical_device_properties =
            unsafe { instance.get_physical_device_format_properties(physical_device, format) };
        if !physical_device_properties
            .optimal_tiling_features
            .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
        {
            return Err(VulkanError::ImageLinearBlittingNotSupported);
        }

        let command_buffer = begin_single_time_commands(logical_device, command_pool)?;

        let mut memory_barrier = vk::ImageMemoryBarrier {
            image: image.image,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            subresource_range: vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1)
                .level_count(1)
                .build(),
            ..Default::default()
        };

        let mut prev_mip_width = i32::try_from(image.width)?;
        let mut prev_mip_height = i32::try_from(image.height)?;

        for cur_mip_level in 1..image.mip_levels {
            let cur_mip_width = if prev_mip_width > 1 {
                prev_mip_width / 2
            } else {
                1
            };
            let cur_mip_height = if prev_mip_height > 1 {
                prev_mip_height / 2
            } else {
                1
            };
            let prev_mip_level = cur_mip_level - 1;

            // prepare reusable barrier for blit to read from previous mip
            memory_barrier.subresource_range.base_mip_level = prev_mip_level;
            memory_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            memory_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            memory_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            memory_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            // this will now block until the transfer write happens either from previous blit of from copy buffer
            unsafe {
                logical_device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[memory_barrier],
                );
            }

            let mut image_blit_builder = vk::ImageBlit::builder();
            // the two corners of the space to blit from
            image_blit_builder.src_offsets = [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: prev_mip_width,
                    y: prev_mip_height,
                    z: 1,
                },
            ];
            image_blit_builder.src_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(prev_mip_level)
                .base_array_layer(0)
                .layer_count(1)
                .build();
            // the two corners of the space to blit to
            image_blit_builder.dst_offsets = [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: cur_mip_width,
                    y: cur_mip_height,
                    z: 1,
                },
            ];
            image_blit_builder.dst_subresource = vk::ImageSubresourceLayers::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .mip_level(cur_mip_level)
                .base_array_layer(0)
                .layer_count(1)
                .build();

            // do the mip -> mip downscale blit
            unsafe {
                logical_device.cmd_blit_image(
                    command_buffer,
                    image.image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[image_blit_builder.build()],
                    vk::Filter::LINEAR,
                );
            }

            // prepare reusable barrier to block until layer transitions into a shader read optimal layout
            // specify that this transition can happen after the transfer read that was reading to do the mip -> mip downscale blit
            memory_barrier.subresource_range.base_mip_level = prev_mip_level;
            memory_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            memory_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            memory_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            memory_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                logical_device.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[memory_barrier],
                );
            }

            prev_mip_width = cur_mip_width;
            prev_mip_height = cur_mip_height;
        }

        // we need one more transition since the last mip level doesn't get handled by the loop
        memory_barrier.subresource_range.base_mip_level = image.mip_levels - 1;
        memory_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        memory_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        memory_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        memory_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            logical_device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[memory_barrier],
            );
        }

        end_single_time_commands(command_buffer, logical_device, queues, command_pool)
    }
}
