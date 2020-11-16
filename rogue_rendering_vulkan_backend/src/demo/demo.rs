use rogue_rendering_vulkan_backend::devices::logical_device::create_logical_device;
use rogue_rendering_vulkan_backend::devices::physical_device::pick_physical_device;
use rogue_rendering_vulkan_backend::devices::queues::{
    create_queues, QueueFamilyIndices, QueueType,
};
use rogue_rendering_vulkan_backend::devices::requirements::DeviceRequirements;
use rogue_rendering_vulkan_backend::drawing::synchronization::SynchronizationContainer;
use rogue_rendering_vulkan_backend::drawing::{command_buffers, framebuffers};
use rogue_rendering_vulkan_backend::graphics_pipeline::GraphicsPipeline;
use rogue_rendering_vulkan_backend::presentation::image_views::ImageViews;
use rogue_rendering_vulkan_backend::presentation::swap_chain::{
    SwapChainContainer, SwapChainSupportDetails,
};
use rogue_rendering_vulkan_backend::util;
use rogue_rendering_vulkan_backend::util::debug::VulkanDebug;
use rogue_rendering_vulkan_backend::util::platform::SurfaceContainer;
use rogue_rendering_vulkan_backend::util::result::{Result, VulkanError};
use rogue_rendering_vulkan_backend::util::validation::VulkanValidation;

use rustylog::{log, Log};

use ash::prelude::VkResult;
use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CString;
use std::ptr;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const WINDOW_TITLE: &'static str = "Vulkan Demo";
const ENGINE_NAME: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const REQUIRED_QUEUES: [QueueType; 2] = [
    QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS),
    QueueType::PresentQueue,
];
const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

fn is_swap_chain_adequate(swap_chain_details: &SwapChainSupportDetails) -> bool {
    !swap_chain_details.formats.is_empty() && !swap_chain_details.present_modes.is_empty()
}

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    _validation: VulkanValidation,
    debug: VulkanDebug,
    surface_container: SurfaceContainer,
    physical_device: vk::PhysicalDevice,
    logical_device: ash::Device,
    queue_indices: QueueFamilyIndices,
    queues: HashMap<QueueType, ash::vk::Queue>,
    swap_chain_container: SwapChainContainer,
    image_views_container: ImageViews,
    graphics_pipeline: GraphicsPipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    sync_container: SynchronizationContainer,
    buffer_resized: bool,
    buffer_minimized: bool,
}

enum ResizeDetectedLocation { InAcquire, InPresent }

impl VulkanApp {
    fn new(window: &Window) -> Self {
        let entry = ash::Entry::new().unwrap();
        let validation = VulkanValidation::enabled(util::validation::ValidationOptions::Verbose);
        // creating the instance is equivalent to initializing the vulkan library
        let instance =
            Self::create_instance(&entry, &validation).expect("Failed to create instance");
        let debug = VulkanDebug::new(&entry, &instance, &validation);
        // creating a surface to present images to
        let surface_container = util::platform::create_surface(&entry, &instance, &window)
            .expect("Failed to create surface");
        // pick the first graphics card that supports all the features we specified in instance
        let requirements =
            DeviceRequirements::new(&REQUIRED_QUEUES, &DEVICE_EXTENSIONS, is_swap_chain_adequate);
        let physical_device = pick_physical_device(&instance, &surface_container, &requirements)
            .expect("Failed to create physical device");
        // create logical device and queues
        let queue_indices = QueueFamilyIndices::find(
            &instance,
            physical_device,
            &surface_container,
            &requirements,
        )
        .expect("Failed to create queue indices");
        let logical_device = create_logical_device(
            &instance,
            physical_device,
            &queue_indices,
            &requirements,
            &validation,
        )
        .expect("Failed to create logical device");

        let sync_container =
            SynchronizationContainer::create(&logical_device).expect("Failed to create semaphores");

        let command_pool = command_buffers::create_command_pool(&logical_device, &queue_indices)
            .expect("Failed to create command pool");

        let queues = create_queues(&queue_indices, &logical_device).expect("Failed to get queues");

        let (
            swap_chain_container,
            image_views_container,
            graphics_pipeline,
            framebuffers,
            command_buffers,
        ) = Self::create_swapchain_graphics_pipeline_framebuffers_and_command_buffers(
            &instance,
            physical_device,
            &logical_device,
            &queue_indices,
            &surface_container,
            &command_pool,
            &window,
        );

        let result = Self {
            _entry: entry,
            instance,
            _validation: validation,
            debug,
            surface_container,
            physical_device,
            logical_device,
            queue_indices,
            queues,
            swap_chain_container,
            image_views_container,
            graphics_pipeline,
            framebuffers,
            command_pool,
            command_buffers,
            sync_container,
            buffer_resized: false,
            buffer_minimized: false,
        };

        result
    }

    pub fn create_swapchain_graphics_pipeline_framebuffers_and_command_buffers(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        logical_device: &ash::Device,
        queue_indices: &QueueFamilyIndices,
        surface_container: &SurfaceContainer,
        command_pool: &vk::CommandPool,
        window: &Window,
    ) -> (
        SwapChainContainer,
        ImageViews,
        GraphicsPipeline,
        Vec<vk::Framebuffer>,
        Vec<vk::CommandBuffer>,
    ) {
        let swap_chain_container = SwapChainContainer::create(
            &instance,
            physical_device,
            &logical_device,
            &surface_container,
            &window,
            &queue_indices,
        )
        .expect("Failed to create swap chain");

        let image_views_container = ImageViews::create(&logical_device, &swap_chain_container)
            .expect("Failed to create image views");

        let graphics_pipeline = GraphicsPipeline::create(&logical_device, &swap_chain_container)
            .expect("Failed to create graphics pipeline");

        let framebuffers = framebuffers::create_framebuffers(
            &logical_device,
            &graphics_pipeline,
            &image_views_container,
            &swap_chain_container,
        )
        .expect("Failed to create framebuffers");

        // command buffers are released when we destroy the pool
        let command_buffers = command_buffers::create_command_buffers(
            &logical_device,
            &command_pool,
            &framebuffers,
            &graphics_pipeline,
            &swap_chain_container,
        )
        .expect("Failed to create command buffers");

        (
            swap_chain_container,
            image_views_container,
            graphics_pipeline,
            framebuffers,
            command_buffers,
        )
    }

    pub fn recreate_swap_chain(&mut self, window: &Window) -> Result<()> {
        unsafe {
            self.logical_device.device_wait_idle()?;
            self.cleanup_swap_chain();
        }

        let (
            swap_chain_container,
            image_views_container,
            graphics_pipeline,
            framebuffers,
            command_buffers,
        ) = Self::create_swapchain_graphics_pipeline_framebuffers_and_command_buffers(
            &self.instance,
            self.physical_device,
            &self.logical_device,
            &self.queue_indices,
            &self.surface_container,
            &self.command_pool,
            window,
        );

        self.swap_chain_container = swap_chain_container;
        self.image_views_container = image_views_container;
        self.graphics_pipeline = graphics_pipeline;
        self.framebuffers = framebuffers;
        self.command_buffers = command_buffers;

        Ok(())
    }

    fn handle_resize<A>(&mut self, location: ResizeDetectedLocation, result: &VkResult<A>, window: &Window) -> Result<bool> {
        let resize_needed = match location {
            ResizeDetectedLocation::InAcquire => match result {
                Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => true,                
                Err(error) => return Err(VulkanError::VkError(error.to_string())),
                Ok(_) => false,
            },
            ResizeDetectedLocation::InPresent => match result {
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => true,
            Err(error) => return Err(VulkanError::VkError(error.to_string())),
            // if a window event signaled that a resize happened then we want to handle the resize after image present
            Ok(_) => self.buffer_resized,
            },
        };
        let resize_happened = if resize_needed {
            self.buffer_resized = false;
            self.recreate_swap_chain(window)?;
            true
        } else {
            false
        };
        Ok(resize_happened)
    }

    fn draw_frame(&mut self, window: &Window) -> Result<()> {
        if self.buffer_minimized {
            return Ok(());
        }

        let cpu_gpu_to_wait_for = [self.sync_container.get_in_flight_fence()];
        unsafe {
            self.logical_device
                .wait_for_fences(&cpu_gpu_to_wait_for, true, u64::MAX)?;
        }

        // get an available image from the swapchain
        let timeout = u64::MAX;
        let acquire_result = unsafe {
            self.swap_chain_container
                .swap_chain_loader
                .acquire_next_image(
                    self.swap_chain_container.swap_chain,
                    timeout,
                    self.sync_container.get_image_available_semaphore(),
                    vk::Fence::null(),
                )
        };
        if self.handle_resize(ResizeDetectedLocation::InAcquire, &acquire_result, window)? {
            return Ok(());
        }
        let (available_image_index_u32, _) = acquire_result?;
        let available_image_index = usize::try_from(available_image_index_u32)?;

        // wait on fence to see if image isn't being used already by an in-flight frame
        if self
            .sync_container
            .get_image_in_flight_fence(available_image_index)
            != vk::Fence::null()
        {
            let image_fence = [self
                .sync_container
                .get_image_in_flight_fence(available_image_index)];
            unsafe {
                self.logical_device
                    .wait_for_fences(&image_fence, true, u64::MAX)?;
            }
        }
        // save the fence that will be used for the image used by this in-flight frame
        self.sync_container.set_image_in_flight_fence(
            available_image_index,
            self.sync_container.get_in_flight_fence(),
        );

        // specify that we want to delay the execution of the submit of the command buffer
        // specificially, we want to wait until the wiriting to the color attachment is done on the available image
        let wait_semaphores = [self.sync_container.get_image_available_semaphore()];
        let wait_semaphores_count = u32::try_from(wait_semaphores.len())?;
        let wait_stages: Vec<_> = wait_semaphores
            .iter()
            .map(|_x| vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .collect();

        let command_buffer_ptr = if !self.command_buffers.is_empty() {
            &self.command_buffers[available_image_index]
        } else {
            return Err(VulkanError::CommandBufferNotAvailable(
                available_image_index,
            ));
        };

        let signal_semaphores = [self.sync_container.get_render_finished_semaphore()];
        let signal_semaphores_count = u32::try_from(signal_semaphores.len())?;

        let command_buffer_submit_infos = [vk::SubmitInfo {
            wait_semaphore_count: wait_semaphores_count,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: command_buffer_ptr,
            signal_semaphore_count: signal_semaphores_count,
            p_signal_semaphores: signal_semaphores.as_ptr(),
            ..Default::default()
        }];

        let graphics_queue = self.get_graphics_queue()?;
        let cpu_gpu_fence = self.sync_container.get_in_flight_fence();
        unsafe {
            self.logical_device.reset_fences(&[cpu_gpu_fence])?;
            self.logical_device.queue_submit(
                graphics_queue,
                &command_buffer_submit_infos,
                cpu_gpu_fence,
            )?
        }

        // present the image to swap chain
        let swap_chains = [self.swap_chain_container.swap_chain];
        let swap_chain_count = u32::try_from(swap_chains.len())?;
        let present_info = vk::PresentInfoKHR {
            wait_semaphore_count: signal_semaphores_count,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            p_swapchains: swap_chains.as_ptr(),
            swapchain_count: swap_chain_count,
            p_image_indices: &available_image_index_u32,
            ..Default::default()
        };

        let present_queue = self.get_present_queue()?;
        let present_result = unsafe {
            self.swap_chain_container
                .swap_chain_loader
                .queue_present(present_queue, &present_info)
        };

        let _resize_happened = self.handle_resize(ResizeDetectedLocation::InPresent, &present_result, window)?;

        self.sync_container.update_frame_counter();

        Ok(())
    }

    fn wait_until_device_idle(&self) -> Result<()> {
        unsafe {
            self.logical_device.device_wait_idle()?;
        }
        Ok(())
    }

    fn get_graphics_queue(&self) -> Result<ash::vk::Queue> {
        self.queues
            .get(&QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS))
            .map(|&x| x)
            .ok_or(VulkanError::QueueGraphicsNotFound)
    }

    fn get_present_queue(&self) -> Result<ash::vk::Queue> {
        self.queues
            .get(&QueueType::PresentQueue)
            .map(|&x| x)
            .ok_or(VulkanError::QueuePresentNotFound)
    }

    fn create_instance(entry: &ash::Entry, validation: &VulkanValidation) -> Result<ash::Instance> {
        if validation.check_validation_layer_support(entry)? == false {
            return Err(VulkanError::RequiredValidationLayersUnsupported);
        }

        let app_name = CString::new(WINDOW_TITLE).unwrap();
        let engine_name = CString::new(ENGINE_NAME).unwrap();
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            p_application_name: app_name.as_ptr(),
            application_version: vk::make_version(1, 0, 0),
            p_engine_name: engine_name.as_ptr(),
            engine_version: vk::make_version(1, 0, 0),
            api_version: vk::make_version(1, 0, 0),
        };

        let extension_names = util::platform::required_extension_names();

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: VulkanDebug::get_creation_destruction_debug_create_info(validation),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            enabled_layer_count: validation.get_enabled_layer_count(),
            pp_enabled_layer_names: validation.get_enabled_layer_names(),
            pp_enabled_extension_names: extension_names.as_ptr(),
            enabled_extension_count: extension_names.len() as u32,
        };

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .expect("Failed to create instance!")
        };

        Ok(instance)
    }

    fn init_window(event_loop: &EventLoop<()>) -> Window {
        winit::window::WindowBuilder::new()
            .with_title(WINDOW_TITLE)
            .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    unsafe fn cleanup_swap_chain(&self) {
        for framebuffer in self.framebuffers.iter() {
            self.logical_device.destroy_framebuffer(*framebuffer, None);
        }

        self.logical_device.free_command_buffers(self.command_pool, &self.command_buffers);

        self.logical_device
            .destroy_pipeline(self.graphics_pipeline.pipeline, None);
        self.logical_device
            .destroy_pipeline_layout(self.graphics_pipeline.pipeline_layout, None);

        self.logical_device
            .destroy_render_pass(self.graphics_pipeline.render_pass, None);

        for &image_view in &self.image_views_container.image_views {
            self.logical_device.destroy_image_view(image_view, None);
        }
        self.swap_chain_container
            .swap_chain_loader
            .destroy_swapchain(self.swap_chain_container.swap_chain, None);
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        log!(Log::Info, "VulkanApp exiting");
        unsafe {
            self.cleanup_swap_chain();

            self.sync_container.destroy(&self.logical_device);
            self.logical_device
                .destroy_command_pool(self.command_pool, None);

            self.logical_device.destroy_device(None);
            self.surface_container
                .surface_loader
                .destroy_surface(self.surface_container.surface, None);
            self.debug.destroy_debug_messenger();
            self.instance.destroy_instance(None);
        }
    }
}

struct Main;

impl Main {
    fn main_loop(mut vulkan_app: VulkanApp, event_loop: EventLoop<()>, window: Window) {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    Self::exit(control_flow);
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            Self::exit(control_flow);
                        }
                        _ => {}
                    },
                },
                WindowEvent::Resized(winit::dpi::PhysicalSize{width, height}) => {
                    log!(Log::Info, "Window was resized");
                    vulkan_app.buffer_resized = true;
                    if width == 0 || height == 0 {
                        log!(Log::Info, "Window was minimized");
                        vulkan_app.buffer_minimized = true;
                    } else {
                        vulkan_app.buffer_minimized = false;
                    }
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                let frame_result = vulkan_app.draw_frame(&window);
                if let Err(error) = frame_result {
                    log!(Log::Error, "Failed to draw frame: {}", error);
                }
            }
            Event::LoopDestroyed => {
                log!(Log::Info, "In exit main loop");
                let wait_result = vulkan_app.wait_until_device_idle();
                if let Err(error) = wait_result {
                    log!(
                        Log::Error,
                        "Failed while waiting until device idle: {}",
                        error
                    );
                }
            }
            _ => {}
        });
    }

    fn exit(control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Exit
    }
}

fn main() {
    log!(Log::Info, "Hello demo");
    let event_loop = EventLoop::new();
    let window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new(&window);

    Main::main_loop(vulkan_app, event_loop, window);
}
