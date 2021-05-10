use log::info;
use mimic_frontend::{main_loop::MainLoop, winit_window};
use mimic_vulkan_backend::backend::mimic_backend::VulkanApp;
use rustyutil::apptime::AppTime;

const WINDOW_TITLE: &'static str = "Vulkan Demo";
const ENGINE_NAME: &'static str = "Vulkan Engine";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    env_logger::init();
    info!("Hello demo");
    // let event_loop = EventLoop::new();
    // let winit_window =
    //     MainLoop::init_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT, &event_loop);

    // let window_surface = winit_window::get_window_surface_from_winit(&winit_window)
    //     .expect("Failed to get window surface");
    // let window_size =
    //     winit_window::get_window_size_from_winit(&winit_window).expect("Failed to get window size");

    // let vulkan_app = VulkanApp::new(WINDOW_TITLE, ENGINE_NAME, &window_surface, &window_size);

    // let apptime = AppTime::new();
    // MainLoop::run(vulkan_app, event_loop, winit_window, apptime);
}
