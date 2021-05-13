use log::info;
use mimic_frontend::main_loop::MainLoop;

const WINDOW_TITLE: &'static str = "Vulkan Demo";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

fn main() {
    env_logger::init();
    info!("Hello demo");
    MainLoop::run(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT);
}
