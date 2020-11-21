use crate::util::result::Result;

use ash::vk;
use memoffset::offset_of;
use std::convert::TryFrom;

pub struct Vertex {
    pub pos: glm::Vec2,
    pub color: glm::Vec3,
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

    pub fn get_attribute_descriptions() -> Result<[vk::VertexInputAttributeDescription; 2]> {
        let mut attribute_descriptions: [vk::VertexInputAttributeDescription; 2] =
            [Default::default(); 2];

        // vertex shader location 0 -> position
        attribute_descriptions[0].binding = 0;
        attribute_descriptions[0].location = 0;
        attribute_descriptions[0].format = vk::Format::R32G32_SFLOAT;
        attribute_descriptions[0].offset = u32::try_from(offset_of!(Vertex, pos))?;

        // vertex shader location 1 -> color
        attribute_descriptions[1].binding = 0;
        attribute_descriptions[1].location = 1;
        attribute_descriptions[1].format = vk::Format::R32G32B32_SFLOAT;
        attribute_descriptions[1].offset = u32::try_from(offset_of!(Vertex, color))?;

        Ok(attribute_descriptions)
    }
}
