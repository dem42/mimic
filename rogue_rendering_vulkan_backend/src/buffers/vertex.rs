use crate::buffers::memory::MemoryCopyable;
use crate::util::result::Result;

use ash::vk;
use memoffset::offset_of;
use std::convert::TryFrom;

#[repr(C)]
pub struct Vertex {
    pub pos: glm::Vec3,
    pub color: glm::Vec3,
    pub tex_coord: glm::Vec2,
}

impl Vertex {
    pub fn get_binding_description() -> Result<vk::VertexInputBindingDescription> {
        Ok(vk::VertexInputBindingDescription {
            // index of the binding in the array of bindings
            binding: 0,
            stride: u32::try_from(std::mem::size_of::<Self>())?,
            input_rate: vk::VertexInputRate::VERTEX,
            ..Default::default()
        })
    }

    pub fn get_attribute_descriptions() -> Result<[vk::VertexInputAttributeDescription; 3]> {
        Ok([
            vk::VertexInputAttributeDescription {
                binding: 0,
                // vertex shader location 0 -> position
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: u32::try_from(offset_of!(Vertex, pos))?,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                // vertex shader location 1 -> color
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: u32::try_from(offset_of!(Vertex, color))?,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                // vertex shader location 2 -> texture coords
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: u32::try_from(offset_of!(Vertex, tex_coord))?,
            },
        ])
    }
}

impl MemoryCopyable for [Vertex] {
    unsafe fn copy_to_mapped_memory(&self, data_target_ptr: *mut std::ffi::c_void) {
        let data_ptr = data_target_ptr as *mut Vertex;
        data_ptr.copy_from_nonoverlapping(self.as_ptr(), self.len());
    }
}
