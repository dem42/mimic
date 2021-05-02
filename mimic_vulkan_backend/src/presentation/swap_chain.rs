use crate::devices::queues::QueueFamilyIndices;
use crate::util::platform::SurfaceContainer;
use crate::util::result::{Result, VulkanError};
use crate::window::window::WindowSize;

use ash::extensions::khr;
use ash::vk;
use std::cmp;
use std::convert::TryFrom;

pub struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl SwapChainSupportDetails {
    pub fn query_support(
        physical_device: vk::PhysicalDevice,
        surface: &SurfaceContainer,
    ) -> Result<Self> {
        let surface_capabilities = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface.surface)?
        };
        let formats = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_formats(physical_device, surface.surface)?
        };
        let present_modes = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_present_modes(physical_device, surface.surface)?
        };

        Ok(Self {
            capabilities: surface_capabilities,
            formats,
            present_modes,
        })
    }

    // swap extent is the resolution of the images we are writing to
    fn choose_swap_extent(&self, window_size: &WindowSize) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            return self.capabilities.current_extent;
        } else {
            let actual_extent = vk::Extent2D {
                width: window_size.monitor_resolution_width,
                height: window_size.monitor_resolution_height,
            };
            let result_extent = vk::Extent2D {
                width: cmp::max(
                    self.capabilities.min_image_extent.width,
                    cmp::min(
                        self.capabilities.max_image_extent.width,
                        actual_extent.width,
                    ),
                ),
                height: cmp::max(
                    self.capabilities.min_image_extent.height,
                    cmp::min(
                        self.capabilities.max_image_extent.height,
                        actual_extent.height,
                    ),
                ),
            };
            result_extent
        }
    }

    fn choose_swap_surface_present(&self) -> vk::PresentModeKHR {
        for &present_mode in &self.present_modes {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                return vk::PresentModeKHR::MAILBOX;
            }
        }
        // fifo always guaranteed to be available
        return vk::PresentModeKHR::FIFO;
    }

    fn choose_swap_surface_format(&self) -> Result<vk::SurfaceFormatKHR> {
        for format in &self.formats {
            // non-linear sRGB looks the best to the human eye because
            // human eyes are better at telling apart low frequences (darker colors)
            // but worse at telling high frequencies so if we just use a linear RGB
            // we would end up with banding on low frequencies (jarring transitions between colors)
            // and colors that look the same on high frequencies
            // with a non-linear color space we avoid this problem
            // NOTE that linear RGB does have an advantage when it comes to shading because 2* a frequencies makes it twice as light
            // which means you can maniuplate such colors with shaders nicely
            if format.format == vk::Format::B8G8R8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            {
                return Ok(*format);
            }
        }

        if self.formats.is_empty() {
            Err(VulkanError::SwapChainFormatsError)
        } else {
            Ok(self.formats[0])
        }
    }

    fn choose_swap_min_image_count(&self) -> u32 {
        let image_count = self.capabilities.min_image_count + 1;
        if self.capabilities.max_image_count > 0 && self.capabilities.max_image_count < image_count
        {
            self.capabilities.max_image_count
        } else {
            image_count
        }
    }
}

pub struct SwapChainContainer {
    pub swap_chain_loader: khr::Swapchain,
    pub swap_chain: vk::SwapchainKHR,
    pub swap_chain_images: Vec<vk::Image>,
    pub swap_chain_format: vk::SurfaceFormatKHR,
    pub swap_chain_extent: vk::Extent2D,
}

impl SwapChainContainer {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        surface: &SurfaceContainer,
        window_size: &WindowSize,
        queue_indices: &QueueFamilyIndices,
    ) -> Result<Self> {
        let swap_chain_support_details =
            SwapChainSupportDetails::query_support(physical_device, surface)?;

        let surface_format = swap_chain_support_details.choose_swap_surface_format()?;
        let present_mode = swap_chain_support_details.choose_swap_surface_present();
        let extent = swap_chain_support_details.choose_swap_extent(window_size);
        let min_image_count = swap_chain_support_details.choose_swap_min_image_count();

        let (sharing_mode, sharing_queue_count, sharing_indices) =
            queue_indices.get_image_sharing_details();

        let swap_chain_create_info = vk::SwapchainCreateInfoKHR {
            surface: surface.surface,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            image_extent: extent,
            min_image_count,
            present_mode,
            clipped: vk::TRUE,
            image_array_layers: 1,
            image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
            image_sharing_mode: sharing_mode,
            queue_family_index_count: u32::try_from(sharing_queue_count)?,
            p_queue_family_indices: sharing_indices.as_ptr(),
            pre_transform: swap_chain_support_details.capabilities.current_transform,
            composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
            ..vk::SwapchainCreateInfoKHR::default()
        };

        let swap_chain_loader = khr::Swapchain::new(instance, logical_device);
        let swap_chain =
            unsafe { swap_chain_loader.create_swapchain(&swap_chain_create_info, None)? };

        let swap_chain_images = unsafe { swap_chain_loader.get_swapchain_images(swap_chain)? };

        Ok(SwapChainContainer {
            swap_chain,
            swap_chain_loader,
            swap_chain_images,
            swap_chain_format: surface_format,
            swap_chain_extent: extent,
        })
    }
}
