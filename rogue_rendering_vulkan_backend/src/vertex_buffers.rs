pub mod memory;
pub mod vertex;

use crate::util::result::Result;
use crate::vertex_buffers::vertex::Vertex;

use ash::version::DeviceV1_0;
use ash::vk;
use std::convert::TryFrom;

pub struct VertexBuffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
    pub vertex_count: usize,
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

impl VertexBuffer {
    pub fn create(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
    ) -> Result<Self> {
        let simple_triangle = get_simple_triangle();
        let vertex_count = simple_triangle.len();

        let buffer_create_info = vk::BufferCreateInfo {
            size: vk::DeviceSize::try_from(std::mem::size_of_val(&simple_triangle))?,
            usage: vk::BufferUsageFlags::VERTEX_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };

        let vertex_buffer = unsafe { logical_device.create_buffer(&buffer_create_info, None)? };

        let vertex_buffer_memory =
            memory::create_device_memory(instance, physical_device, logical_device, vertex_buffer)?;

        unsafe {
            // IMPORTANT: this only works with array slices due to the copy_non_overlapping 
            // it does not work with data being a Vec
            memory::fill_vertex_buffer(logical_device, vertex_buffer_memory, &simple_triangle)?;
        }

        Ok(Self {
            buffer: vertex_buffer,
            memory: vertex_buffer_memory,
            vertex_count,
        })
    }
}
