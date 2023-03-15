use env_logger::fmt::Color;
use log::{info, Level};
use mimic_common::{
    apptime::AppTime,
    config::MimicConfig,
    uniforms::{copy_uniform_to_memory, UniformBufferObject, UniformSpec},
};
use mimic_frontend::{
    cameras::camera::Camera,
    main_loop::{Application, MainLoopBuilder},
    render_commands::RenderCommands,
};
use nalgebra_glm as glm;
use std::{io::Write, path::PathBuf, rc::Rc};
//////////////////////// Consts ///////////////////////
const WINDOW_TITLE: &'static str = "Vulkan Demo";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
//////////////////////// Structs ///////////////////////
struct Material {
    texture_file: Rc<PathBuf>,
    model: Rc<PathBuf>,
    vertex_shader: Rc<PathBuf>,
    frag_shader: Rc<PathBuf>,
}

struct GameObject {
    material: Material,
    position: glm::Vec3,
    rotation: glm::Mat4,
}

#[derive(Default)]
struct Scene {
    is_loaded: bool,
    camera: Camera,
    game_objects: Vec<GameObject>,
}

#[derive(Default)]
struct SceneManager {
    scenes: Vec<Scene>,
}

struct Demo {
    current_scene: Option<usize>,
    scene_manager: SceneManager,
}

struct DemoUniformSpec {
    camera: Camera,
    static_model_transform: glm::Mat4,
}
//////////////////////// Impls ///////////////////////
impl Scene {
    fn load(&mut self, config: &MimicConfig) {
        self.is_loaded = true;

        self.camera = Camera::new(
            glm::vec3(0., 0., 3.),
            glm::vec3(0., 0., -1.),
            glm::vec3(0., 1., 0.),
        );

        // add game objects
        {
            let material = Material {
                texture_file: Rc::new(config.resolve_resource("res/textures/texture.jpg").unwrap()),
                model: Rc::new(config.resolve_resource("res/models/quad.obj").unwrap()),
                vertex_shader: Rc::new(
                    config
                        .resolve_resource("res/shaders/spv/cube.vert.spv")
                        .unwrap(),
                ),
                frag_shader: Rc::new(
                    config
                        .resolve_resource("res/shaders/spv/quad_textured.frag.spv")
                        .unwrap(),
                ),
            };
            let quad_go = GameObject {
                material,
                position: glm::zero(),
                rotation: glm::rotation(std::f32::consts::FRAC_PI_2, &glm::Vec3::x_axis()),
            };
            self.game_objects.push(quad_go);
        }
    }
}

impl Demo {
    fn new() -> Self {
        let scenes = vec![Scene::default()];
        let scene_manager = SceneManager { scenes };
        Demo {
            current_scene: None,
            scene_manager,
        }
    }
}

impl Application for Demo {
    fn update(&mut self, _apptime: &AppTime, config: &MimicConfig) -> RenderCommands {
        let mut render_commands = RenderCommands::default();
        let mut should_redraw = true;

        if let None = self.current_scene {
            should_redraw = true;
            let new_scene = 0;
            if new_scene < self.scene_manager.scenes.len() {
                if !self.scene_manager.scenes[new_scene].is_loaded {
                    self.scene_manager.scenes[new_scene].load(config);
                }

                for go in self.scene_manager.scenes[new_scene].game_objects.iter() {
                    render_commands.draw_textured_model(
                        &go.material.texture_file,
                        &go.material.model,
                        &go.material.vertex_shader,
                        &go.material.frag_shader,
                        Box::new(DemoUniformSpec::new(
                            self.scene_manager.scenes[new_scene].camera.clone(),
                            go.rotation,
                        )),
                    );
                }
                self.current_scene = Some(new_scene);
            }
        }
        render_commands.request_redraw = should_redraw;
        render_commands
    }
}

impl DemoUniformSpec {
    fn new(camera: Camera, static_model_transform: glm::Mat4) -> Self {
        Self {
            camera,
            static_model_transform,
        }
    }
}

impl UniformSpec for DemoUniformSpec {
    fn get_uniform_data(
        &self,
        input: mimic_common::uniforms::UniformUpdateInput<'_>,
        memory_target_ptr: *mut core::ffi::c_void,
    ) {
        let width = input.swapchain_image_width as f32;
        let height = input.swapchain_image_height as f32;
        let proj = self.camera.get_projection_matrix(width, height);
        let view = self.camera.get_view_matrix();
        let model = self.static_model_transform * glm::Mat4::identity();

        let ubo = UniformBufferObject { model, view, proj };

        copy_uniform_to_memory(&ubo, memory_target_ptr);
    }

    fn uniform_buffer_size(&self) -> usize {
        std::mem::size_of::<UniformBufferObject>()
    }
}
//////////////////////// Fns ///////////////////////
fn main() {
    init_logger();
    info!("Hello demo");
    MainLoopBuilder::new()
        .with_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Failed to create window")
        .run(Demo::new());
}

fn init_logger() {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            let level = record.level();
            let mut style = buf.style();
            match record.level() {
                Level::Error => style.set_color(Color::Red),
                Level::Warn => style.set_color(Color::Yellow),
                Level::Info => style.set_color(Color::Green),
                Level::Debug => style.set_color(Color::Blue),
                Level::Trace => style.set_color(Color::Cyan),
            };

            writeln!(
                buf,
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                style.value(level),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .init();
}
