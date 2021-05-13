use log::info;
use mimic_frontend::{
    main_loop::{Application, MainLoopBuilder},
    render_commands::RenderCommands,
};
use rustyutil::apptime::AppTime;

const WINDOW_TITLE: &'static str = "Vulkan Demo";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct Demo;

impl Application for Demo {
    fn update(&mut self, render_commands: &mut RenderCommands, _apptime: &AppTime) {
        render_commands.request_redraw = true;
    }
}

fn main() {
    env_logger::init();
    info!("Hello demo");
    MainLoopBuilder::new()
        .with_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT).expect("Failed to create window")
        .run(Demo{});
}
