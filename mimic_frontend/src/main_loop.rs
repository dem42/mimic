use crate::winit_window;
use log::{error, info};
use mimic_vulkan_backend::backend::mimic_backend::VulkanApp;
use rustyutil::apptime::AppTime;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

/// This struct represent the 3D renderer main loop.
/// It sets up a window and runs the renderer within that window.
pub struct MainLoop;

impl MainLoop {
    const ENGINE_NAME: &'static str = "Vulkan Engine";

    /// Initialize a window with the given `window_tile` and the provided `window_width` and `window_height`. 
    /// The provided `event_loop` is used to detect and react to window events.
    fn init_window(
        window_title: &str,
        window_width: u32,
        window_height: u32,
        event_loop: &EventLoop<()>,
    ) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(event_loop)
            .expect("Failed to create window.")
    }

    /// Run the provided `vulkan_app` inside of the window.
    pub fn run(
        window_title: &str,
        window_width: u32,
        window_height: u32,
    ) {
        let event_loop = EventLoop::new();
        let winit_window = Self::init_window(window_title, window_width, window_height, &event_loop);

        let window_surface = winit_window::get_window_surface_from_winit(&winit_window)
            .expect("Failed to get window surface");
        let window_size =
            winit_window::get_window_size_from_winit(&winit_window).expect("Failed to get window size");

        let mut vulkan_app = VulkanApp::new(window_title, Self::ENGINE_NAME, &window_surface, &window_size);
        let mut apptime = AppTime::new();
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
                WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                    info!("Window was resized");
                    vulkan_app.window_resized = true;
                    if width == 0 || height == 0 {
                        info!("Window was minimized");
                        vulkan_app.window_minimized = true;
                    } else {
                        vulkan_app.window_minimized = false;
                    }
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                winit_window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                let time_update_result = apptime.update();
                if let Err(error) = time_update_result {
                    error!("Failed to update app time: {}", error);
                }

                if let Ok(window_size) = winit_window::get_window_size_from_winit(&winit_window) {
                    let frame_result = vulkan_app.draw_frame(&window_size, &apptime);
                    if let Err(error) = frame_result {
                        error!("Failed to draw frame: {}", error);
                    }
                } else {
                    error!("Failed to draw frame due to window size being unavailable");
                }
            }
            Event::LoopDestroyed => {
                info!("In exit main loop");
                let wait_result = vulkan_app.wait_until_device_idle();
                if let Err(error) = wait_result {
                    error!("Failed while waiting until device idle: {}", error);
                }
            }
            _ => {}
        });
    }

    fn exit(control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Exit
    }
}
