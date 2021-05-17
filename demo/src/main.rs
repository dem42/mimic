use log::info;
use mimic_common::apptime::AppTime;
use mimic_frontend::{
    main_loop::{Application, MainLoopBuilder},
    render_commands::RenderCommands,
};
use std::env;

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
    let current_exe = env::current_exe();
    match current_exe {
        Ok(exe_path) => println!("Current exe path: {}", exe_path.display()),
        Err(error) => println!("Failed to get exe path {}", error),
    }

    env_logger::init();
    info!("Hello demo");
    MainLoopBuilder::new()
        .with_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Failed to create window")
        .run(Demo {});
}
