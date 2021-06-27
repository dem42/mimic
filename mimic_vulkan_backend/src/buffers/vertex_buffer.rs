use crate::{
    buffers::buffer::Buffer, devices::queues::QueueMap, models::vertex::Vertex,
    util::result::Result,
};
use ash::{version::DeviceV1_0, vk};
use std::convert::TryFrom;
//////////////////////// Structs ///////////////////////
#[derive(Default)]
pub struct VertexBuffer {
    pub data: Buffer,
    pub vertex_count: usize,
}
//////////////////////// Impls ///////////////////////
impl VertexBuffer {
    pub fn new(
        data: &[Vertex],
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<Self> {
        let size = vk::DeviceSize::try_from(std::mem::size_of::<Vertex>() * data.len())?;

        let vertex_buffer = Buffer::create_and_fill(
            instance,
            physical_device,
            logical_device,
            command_pool,
            queues,
            size,
            &data,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        )?;

        Ok(Self {
            data: vertex_buffer,
            vertex_count: data.len(),
        })
    }

    pub unsafe fn cleanup(self, logical_device: &ash::Device) {
        logical_device.destroy_buffer(self.data.buffer, None);
        logical_device.free_memory(self.data.memory, None);
    }

    pub fn get_rectangle() -> [Vertex; 8] {
        [
            Vertex {
                pos: glm::vec3(-0.5, 0.0, -0.5),
                color: glm::vec3(1.0, 0.0, 0.0),
                tex_coord: glm::vec2(0.0, 0.0),
            },
            Vertex {
                pos: glm::vec3(0.5, 0.0, -0.5),
                color: glm::vec3(0.0, 1.0, 0.0),
                tex_coord: glm::vec2(1.0, 0.0),
            },
            Vertex {
                pos: glm::vec3(0.5, 0.0, 0.5),
                color: glm::vec3(0.0, 0.0, 1.0),
                tex_coord: glm::vec2(1.0, 1.0),
            },
            Vertex {
                pos: glm::vec3(-0.5, 0.0, 0.5),
                color: glm::vec3(1.0, 1.0, 1.0),
                tex_coord: glm::vec2(0.0, 1.0),
            },
            // second rectangle
            Vertex {
                pos: glm::vec3(-0.5, -0.5, -0.5),
                color: glm::vec3(1.0, 0.0, 0.0),
                tex_coord: glm::vec2(0.0, 0.0),
            },
            Vertex {
                pos: glm::vec3(0.5, -0.5, -0.5),
                color: glm::vec3(0.0, 1.0, 0.0),
                tex_coord: glm::vec2(1.0, 0.0),
            },
            Vertex {
                pos: glm::vec3(0.5, -0.5, 0.5),
                color: glm::vec3(0.0, 0.0, 1.0),
                tex_coord: glm::vec2(1.0, 1.0),
            },
            Vertex {
                pos: glm::vec3(-0.5, -0.5, 0.5),
                color: glm::vec3(1.0, 1.0, 1.0),
                tex_coord: glm::vec2(0.0, 1.0),
            },
        ]
    }
}
