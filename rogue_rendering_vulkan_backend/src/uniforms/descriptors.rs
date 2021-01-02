use crate::buffers::buffer::Buffer;
use crate::presentation::swap_chain::SwapChainContainer;
use crate::uniforms::buffers::UniformBufferObject;
use crate::util::result::{Result, VulkanError};

use ash::version::DeviceV1_0;
use ash::vk;
use std::convert::TryFrom;
use std::ptr;

pub fn create_descriptor_set_layout(
    logical_device: &ash::Device,
) -> Result<vk::DescriptorSetLayout> {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX,
        ..Default::default()
    };

    let descriptor_layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 1,
        p_bindings: &ubo_layout_binding,
        ..Default::default()
    };

    let descriptor_layout =
        unsafe { logical_device.create_descriptor_set_layout(&descriptor_layout_info, None)? };

    Ok(descriptor_layout)
}

#[derive(Debug)]
pub struct DescriptorData {
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl DescriptorData {

    pub fn create(
        logical_device: &ash::Device,
        swap_chain_container: &SwapChainContainer,
        descriptor_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<Buffer>,
    ) -> Result<Self> {
        let descriptor_pool = Self::create_descriptor_pool(logical_device, swap_chain_container)?;
        let descriptor_sets = Self::create_descriptor_sets(logical_device, swap_chain_container, descriptor_pool, descriptor_layout, uniform_buffers)?;

        Ok(Self {
            descriptor_pool,
            descriptor_sets,
        })
    }

    fn create_descriptor_pool(
        logical_device: &ash::Device,
        swap_chain_container: &SwapChainContainer,
    ) -> Result<vk::DescriptorPool> {
        let swap_chain_img_cnt = u32::try_from(swap_chain_container.swap_chain_images.len())?;

        let descriptor_pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swap_chain_img_cnt,
            ..Default::default()
        };
        
        let create_info = vk::DescriptorPoolCreateInfo {
            pool_size_count: 1,
            p_pool_sizes: &descriptor_pool_size,
            max_sets: swap_chain_img_cnt,
            ..Default::default()
        };

        let descriptor_pool = unsafe {
            logical_device.create_descriptor_pool(&create_info, None)?
        };

        Ok(descriptor_pool)
    }

    fn create_descriptor_sets(
        logical_device: &ash::Device,
        swap_chain_container: &SwapChainContainer,
        descriptor_pool: vk::DescriptorPool,
        descriptor_layout: vk::DescriptorSetLayout,
        uniform_buffers: &Vec<Buffer>,
    ) -> Result<Vec<vk::DescriptorSet>> {

        let layouts = vec![descriptor_layout; swap_chain_container.swap_chain_images.len()];

        let descriptor_alloc_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            descriptor_set_count: u32::try_from(layouts.len())?,
            p_set_layouts: layouts.as_ptr(),
            ..Default::default()
        };

        let descriptor_sets = unsafe {
            logical_device.allocate_descriptor_sets(&descriptor_alloc_info)?
        };

        if uniform_buffers.len() < swap_chain_container.swap_chain_images.len() {
            return Err(VulkanError::UniformBufferNotAvailable(uniform_buffers.len()));
        }

        for buf_idx in 0..swap_chain_container.swap_chain_images.len() {
            let descriptor_buffer_info = vk::DescriptorBufferInfo {
                offset: 0,
                range: u64::try_from(std::mem::size_of::<UniformBufferObject>())?,
                buffer: uniform_buffers[buf_idx].buffer,
                ..Default::default()
            };

            if buf_idx >= descriptor_sets.len() {
                return Err(VulkanError::DescriptorSetNotAvailable(buf_idx));
            }

            let descriptor_write_info = [vk::WriteDescriptorSet {
                dst_set: descriptor_sets[buf_idx],
                dst_binding: 0,
                dst_array_element: 0,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                p_buffer_info: &descriptor_buffer_info,
                p_image_info: ptr::null(),
                p_texel_buffer_view: ptr::null(),
                ..Default::default()            
            }];

            unsafe {
                logical_device.update_descriptor_sets(&descriptor_write_info, &[]);
            }
        }

        Ok(descriptor_sets)
    }
}