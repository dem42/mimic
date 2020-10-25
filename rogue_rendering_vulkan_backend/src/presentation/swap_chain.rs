use crate::util::result::{Result, VulkanError};
use crate::util::platform::SurfaceContainer;

use ash::vk;

pub struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<vk::SurfaceFormatKHR>,
    present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn query_support(physical_device: vk::PhysicalDevice, surface: &SurfaceContainer) -> Result<Self> {
        let surface_capabilities = unsafe {
            surface.surface_loader.get_physical_device_surface_capabilities(physical_device, surface.surface)?
        };
        let formats = unsafe {
            surface.surface_loader.get_physical_device_surface_formats(physical_device, surface.surface)?
        };
        let present_modes = unsafe {
            surface.surface_loader.get_physical_device_surface_present_modes(physical_device, surface.surface)?
        };

        Ok(Self {
            capabilities: surface_capabilities,
            formats,
            present_modes,
        })
    }
}