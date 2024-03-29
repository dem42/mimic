use crate::{
    render_commands::{RenderCommand, RenderCommands},
    result::Result,
    winit_window,
};
use log::{error, info};
use mimic_common::{apptime::AppTime, config::MimicConfig};
use mimic_vulkan_backend::backend::mimic_backend::VulkanApp;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
//////////////////////// Traits ///////////////////////
pub trait Application {
    fn update(&mut self, apptime: &AppTime, config: &MimicConfig) -> RenderCommands;
}
//////////////////////// Structs ///////////////////////
/// This struct represent the 3D renderer main loop.
/// It sets up a window and runs the renderer within that window.
pub struct MainLoopBuilder {
    event_loop: Option<EventLoop<()>>,
    window: Option<winit::window::Window>,
    vulkan_app: Option<VulkanApp>,
}
//////////////////////// Impls ///////////////////////
impl MainLoopBuilder {
    const ENGINE_NAME: &'static str = "Vulkan Engine";

    pub fn new() -> Self {
        Self {
            event_loop: None,
            window: None,
            vulkan_app: None,
        }
    }

    pub fn with_window(
        &mut self,
        window_title: &str,
        window_width: u32,
        window_height: u32,
    ) -> Result<&mut Self> {
        let mimic_config = MimicConfig::new()?;
        self.event_loop = Some(EventLoop::new());
        self.window = Some(Self::init_window(
            window_title,
            window_width,
            window_height,
            self.event_loop.as_ref().unwrap(),
        )?);

        let window_surface =
            winit_window::get_window_surface_from_winit(self.window.as_ref().unwrap())?;
        let window_size = winit_window::get_window_size_from_winit(self.window.as_ref().unwrap())?;

        let mut vulkan_app = VulkanApp::new(
            window_title,
            Self::ENGINE_NAME,
            &window_surface,
            &window_size,
            mimic_config,
        )?;
        vulkan_app.create_default_render_command()?;
        self.vulkan_app = Some(vulkan_app);

        Ok(self)
    }

    /// Initialize a window with the given `window_tile` and the provided `window_width` and `window_height`.
    /// The provided `event_loop` is used to detect and react to window events.
    fn init_window(
        window_title: &str,
        window_width: u32,
        window_height: u32,
        event_loop: &EventLoop<()>,
    ) -> Result<winit::window::Window> {
        let window = winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(window_width, window_height))
            .build(event_loop)?;
        Ok(window)
    }

    /// Run the provided `vulkan_app` inside of the window.
    /// By restricting the generic type A to be 'static we prevent A from being a reference unless
    /// it is a reference with a 'static lifetime. This means the application is moved into run and later moved into the event loop.
    pub fn run<A: Application + 'static>(&mut self, mut application: A) -> ! {
        let event_loop = self.event_loop.take().unwrap();
        let winit_window = self.window.take().unwrap();
        let mut vulkan_app = self.vulkan_app.take().unwrap();

        let mut apptime = AppTime::new();
        event_loop.run(move |event, _, control_flow| {
            // we set the control flow to poll on every invocation of the event_loop callback
            // this makes it so that after this event_loop iteration finishes another one begins immediately
            // thus there won't be any waiting and we get a call to application.update()
            *control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { event, .. } => {
                    Self::handle_window_event(control_flow, event, &mut vulkan_app);
                }
                Event::MainEventsCleared => {
                    Self::handle_events_cleared(
                        control_flow,
                        &mut apptime,
                        &mut application,
                        &mut vulkan_app,
                        &winit_window,
                    );
                }
                Event::RedrawRequested(_window_id) => {
                    if let Ok(window_size) = winit_window::get_window_size_from_winit(&winit_window)
                    {
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
            }
        })
    }

    fn handle_events_cleared<A: Application + 'static>(
        control_flow: &mut ControlFlow,
        apptime: &mut AppTime,
        application: &mut A,
        vulkan_app: &mut VulkanApp,
        winit_window: &winit::window::Window,
    ) {
        let time_update_result = apptime.update();
        if let Err(error) = time_update_result {
            error!("Failed to update app time: {}", error);
            Self::exit(control_flow);
        } else {
            let mut render_commands = application.update(&apptime, &vulkan_app.resource_resolver);

            for render_command in render_commands.command_queue.drain(..) {
                match render_command {
                    RenderCommand::DrawObject {
                        texture_file,
                        model_file,
                        vertex_shader_file,
                        fragment_shader_file,
                        uniform_spec,
                    } => {
                        let result = vulkan_app.create_render_command(
                            &texture_file,
                            &model_file,
                            &vertex_shader_file,
                            &fragment_shader_file,
                            uniform_spec,
                        );
                        if let Err(error) = result {
                            error!("Failed draw object operation: {}", error);
                            Self::exit(control_flow);
                        }
                    }
                }
            }

            if render_commands.request_redraw {
                winit_window.request_redraw();
            }
        }
    }

    fn handle_window_event(
        control_flow: &mut ControlFlow,
        event: WindowEvent,
        vulkan_app: &mut VulkanApp,
    ) {
        match event {
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
        }
    }

    fn exit(control_flow: &mut ControlFlow) {
        *control_flow = ControlFlow::Exit
    }
}
