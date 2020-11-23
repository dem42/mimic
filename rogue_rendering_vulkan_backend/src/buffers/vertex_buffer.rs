use crate::buffers::buffer::Buffer;

use crate::devices::queues::QueueMap;
use crate::util::result::Result;

use crate::buffers::vertex::Vertex;

use ash::vk;
use std::convert::TryFrom;

pub struct VertexBuffer {
    pub data: Buffer,
    pub vertex_count: usize,
}

impl VertexBuffer {
    pub fn create(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        command_pool: vk::CommandPool,
        queues: &QueueMap,
    ) -> Result<Self> {
        let simple_triangle = Self::get_simple_triangle();
        let size = vk::DeviceSize::try_from(std::mem::size_of_val(&simple_triangle))?;

        let vertex_buffer = Buffer::create_and_fill(instance, physical_device, logical_device, command_pool, queues, size, &simple_triangle)?;

        Ok(Self {
            data: vertex_buffer,
            vertex_count: simple_triangle.len(),
        })
    }

    fn get_simple_triangle() -> [Vertex; 3] {
        [
            Vertex {
                pos: glm::vec2(0.0, -0.5),
                color: glm::vec3(1.0, 1.0, 1.0),
            },
            Vertex {
                pos: glm::vec2(0.5, 0.5),
                color: glm::vec3(0.0, 1.0, 0.0),
            },
            Vertex {
                pos: glm::vec2(-0.5, 0.5),
                color: glm::vec3(0.0, 0.0, 1.0),
            },
        ]
    }

    fn get_rectangle() -> [Vertex; 4] {
        [
            Vertex {
                pos: glm::vec2(-0.5, -0.5),
                color: glm::vec3(1.0, 0.0, 0.0),
            },
            Vertex {
                pos: glm::vec2(0.5, -0.5),
                color: glm::vec3(0.0, 1.0, 0.0),
            },
            Vertex {
                pos: glm::vec2(0.5, 0.5),
                color: glm::vec3(0.0, 0.0, 1.0),
            },
            Vertex {
                pos: glm::vec2(-0.5, 0.5),
                color: glm::vec3(1.0, 1.0, 1.0),
            },
        ]
    }
}