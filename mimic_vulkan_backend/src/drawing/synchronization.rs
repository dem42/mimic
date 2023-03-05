use crate::util::result::Result;

use ash::vk;

use std::collections::HashMap;

pub struct SynchronizationContainer {
    image_available_semaphores: [vk::Semaphore; SynchronizationContainer::MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [vk::Semaphore; SynchronizationContainer::MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [vk::Fence; SynchronizationContainer::MAX_FRAMES_IN_FLIGHT],
    images_in_flight_fences: HashMap<usize, vk::Fence>,
    current_frame_idx: usize,
}

impl SynchronizationContainer {
    const MAX_FRAMES_IN_FLIGHT: usize = 2;

    pub fn create(logical_device: &ash::Device) -> Result<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            ..Default::default()
        };
        let fence_create_info = vk::FenceCreateInfo {
            // create fence in signalled state so it can be used in await before used
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores = [vk::Semaphore::null(); Self::MAX_FRAMES_IN_FLIGHT];
        let mut render_finished_semaphores = [vk::Semaphore::null(); Self::MAX_FRAMES_IN_FLIGHT];
        let mut in_flight_fences =
            [vk::Fence::null(); SynchronizationContainer::MAX_FRAMES_IN_FLIGHT];

        for i in 0..Self::MAX_FRAMES_IN_FLIGHT {
            image_available_semaphores[i] =
                unsafe { logical_device.create_semaphore(&semaphore_create_info, None)? };
            render_finished_semaphores[i] =
                unsafe { logical_device.create_semaphore(&semaphore_create_info, None)? };
            in_flight_fences[i] = unsafe { logical_device.create_fence(&fence_create_info, None)? };
        }

        let images_in_flight_fences = HashMap::new();

        Ok(Self {
            image_available_semaphores,
            render_finished_semaphores,
            in_flight_fences,
            images_in_flight_fences,
            current_frame_idx: 0,
        })
    }

    pub fn update_frame_counter(&mut self) {
        self.current_frame_idx = (self.current_frame_idx + 1) % Self::MAX_FRAMES_IN_FLIGHT
    }

    /// # Safety
    ///
    /// This function calls low level vulkan api to destroy semaphores and fences
    pub unsafe fn destroy(&self, logical_device: &ash::Device) {
        for i in 0..self.image_available_semaphores.len() {
            logical_device.destroy_semaphore(self.image_available_semaphores[i], None);
        }
        for i in 0..self.render_finished_semaphores.len() {
            logical_device.destroy_semaphore(self.render_finished_semaphores[i], None);
        }
        for i in 0..self.in_flight_fences.len() {
            logical_device.destroy_fence(self.in_flight_fences[i], None);
        }
    }

    pub fn get_image_available_semaphore(&self) -> vk::Semaphore {
        self.image_available_semaphores[self.current_frame_idx]
    }

    pub fn get_render_finished_semaphore(&self) -> vk::Semaphore {
        self.render_finished_semaphores[self.current_frame_idx]
    }

    pub fn get_in_flight_fence(&self) -> vk::Fence {
        self.in_flight_fences[self.current_frame_idx]
    }

    pub fn set_image_in_flight_fence(&mut self, image_index: usize, fence: vk::Fence) {
        self.images_in_flight_fences.insert(image_index, fence);
    }

    pub fn get_image_in_flight_fence(&self, image_index: usize) -> vk::Fence {
        let result = self.images_in_flight_fences.get(&image_index);
        match result {
            Some(fence) => *fence,
            None => vk::Fence::null(),
        }
    }
}
