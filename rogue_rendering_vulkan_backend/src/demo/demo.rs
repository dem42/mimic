use rogue_rendering_vulkan_backend::devices::physical_device::pick_physical_device;
use rogue_rendering_vulkan_backend::devices::logical_device::create_logical_device;
use rogue_rendering_vulkan_backend::devices::queues::{create_queues, QueueFamilyIndices, QueueType};
use rogue_rendering_vulkan_backend::devices::requirements::DeviceRequirements;
use rogue_rendering_vulkan_backend::presentation::swap_chain::SwapChainSupportDetails;
use rogue_rendering_vulkan_backend::util;
use rogue_rendering_vulkan_backend::util::debug::VulkanDebug;
use rogue_rendering_vulkan_backend::util::platform::SurfaceContainer;
use rogue_rendering_vulkan_backend::util::result::{Result, VulkanError};
use rogue_rendering_vulkan_backend::util::validation::VulkanValidation;

use rustylog::log;
use rustylog::Log;

use ash::version::{DeviceV1_0, EntryV1_0, InstanceV1_0};
use ash::vk;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

const WINDOW_TITLE: &'static str = "Vulkan Demo";
const ENGINE_NAME: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const REQUIRED_QUEUES: [QueueType; 2] = [QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS), QueueType::PresentQueue];
const DEVICE_EXTENSIONS: [&'static str; 1] = ["VK_KHR_swapchain"];

fn is_swap_chain_adequate(swap_chain_details: &SwapChainSupportDetails) -> bool {
    true
}

struct VulkanApp {
    _entry: ash::Entry,
    instance: ash::Instance,
    _validation: VulkanValidation,
    debug: VulkanDebug,
    surface_container: SurfaceContainer,
    _physical_device: vk::PhysicalDevice,
    logical_device: ash::Device,
    queues: HashMap<QueueType, ash::vk::Queue>,
}

impl VulkanApp {

    fn new(window: &Window) -> Self {
        let entry = ash::Entry::new().unwrap();
        let validation = VulkanValidation::enabled(util::validation::ValidationOptions::Verbose);
        // creating the instance is equivalent to initializing the vulkan library
        let instance = Self::create_instance(&entry, &validation).expect("Failed to create instance");
        let debug = VulkanDebug::new(&entry, &instance, &validation);
        // creating a surface to present images to
        let surface_container = util::platform::create_surface(&entry, &instance, &window).expect("Failed to create surface");
        // pick the first graphics card that supports all the features we specified in instance
        let requirements = DeviceRequirements::new(&REQUIRED_QUEUES, &DEVICE_EXTENSIONS, is_swap_chain_adequate);
        let physical_device = pick_physical_device(&instance, &surface_container, &requirements).expect("Failed to create physical device");
        // create logical device and queues
        let queue_indices = QueueFamilyIndices::find(&instance, physical_device, &surface_container, &requirements).expect("Failed to create queue indices");
        let logical_device = create_logical_device(&instance, physical_device, &queue_indices, &requirements, &validation).expect("Failed to create logical device");
        let queues = create_queues(&queue_indices, &logical_device).expect("Failed to get queues");

        let result = Self {
            _entry : entry,
            instance,
            _validation: validation,
            debug,
            surface_container,
            _physical_device: physical_device,
            logical_device,
            queues,
        };

        result
    }

    fn get_graphics_queue(&self) -> &ash::vk::Queue {
        self.queues.get(&QueueType::QueueWithFlag(vk::QueueFlags::GRAPHICS)).expect("Failed to get graphics queue")
    }

    fn get_present_queue(&self) -> &ash::vk::Queue {
        self.queues.get(&QueueType::PresentQueue).expect("Failed to get present queue")
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
}

impl Drop for VulkanApp {
    fn drop(&mut self) { 
        log!(Log::Info, "VulkanApp exiting");
        unsafe {
            self.logical_device.destroy_device(None);
            self.surface_container.surface_loader.destroy_surface(self.surface_container.surface, None);
            self.debug.destroy_debug_messenger();
            self.instance.destroy_instance(None);
        }
    }
}

struct Main;

impl Main {
    fn main_loop(vulkan_app: VulkanApp, event_loop: EventLoop<()>, window: Window) {

        event_loop.run(move |event, _, control_flow| {
            // using this is kind of a dirty trick because i want to move vulkan_app into the event_loop FnMut callback
            // the reason to do that is because the FnMut closure gets dropped when the event loop exits
            // but because after the event loop exits winit simply std::process:exits which means nothing that hasn't been moved into
            // the callback closure will get Dropped -> so move vulkan app into it by taking a immutable borrow everytime this FnMut is called
            let _vulkan_app_to_drop_when_closure_exits = &vulkan_app;

            match event {
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        },
                        WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                KeyboardInput { virtual_keycode, state, .. } => {
                                    match (virtual_keycode, state) {
                                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        },
                                        _ => {},
                                    }
                                },
                            }
                        },
                        _ => {},
                    }
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                },
                _ => {},
            }
        });
    }
}

fn main() {
    log!(Log::Info, "Hello demo");
    let event_loop = EventLoop::new();
    let window = VulkanApp::init_window(&event_loop);

    let vulkan_app = VulkanApp::new(&window);

    vulkan_app.get_graphics_queue();
    vulkan_app.get_present_queue();

    Main::main_loop(vulkan_app, event_loop, window);
}