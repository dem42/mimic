use log::info;
use mimic_common::{apptime::AppTime, config::MimicConfig, texture::FilesystemTextureSource, uniforms::{copy_uniform_to_memory, UniformBufferObject, UniformSpec}};
use mimic_frontend::{
    cameras::camera::Camera,
    main_loop::{Application, MainLoopBuilder},
    render_commands::RenderCommands,
};
use nalgebra_glm as glm;
//////////////////////// Consts ///////////////////////
const WINDOW_TITLE: &'static str = "Vulkan Demo";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
//////////////////////// Structs ///////////////////////
#[derive(Default)]
struct Demo {
    scene_sent: bool,
}

struct DemoUniformSpec {
    camera: Camera,
    static_model_transform: glm::Mat4,
}
//////////////////////// Impls ///////////////////////
impl Application for Demo {
    fn update(
        &mut self,
        render_commands: &mut RenderCommands,
        apptime: &AppTime,
        config: &MimicConfig,
    ) {
        render_commands.request_redraw = true;

        if !self.scene_sent && apptime.elapsed_since_game_start.as_secs_f32() > 2.0 {

            let rot: glm::Mat4 = glm::rotation(std::f32::consts::FRAC_PI_2, &glm::Vec3::x_axis());

            let texture_source = Box::new(
                FilesystemTextureSource::new(config.resolve_resource("res/textures/texture.jpg").unwrap()).unwrap()
            );

            render_commands.draw_textured_model(
                texture_source,
                config.resolve_resource("res/models/quad.obj").unwrap(),
                config
                    .resolve_resource("res/shaders/spv/cube.vert.spv")
                    .unwrap(),
                config
                    .resolve_resource("res/shaders/spv/quad_textured.frag.spv")
                    .unwrap(),
                Box::new(DemoUniformSpec::new(
                    Camera::new(
                        glm::vec3(0., 0., 3.),
                        glm::vec3(0., 0., -1.),
                        glm::vec3(0., 1., 0.),
                    ),
                    rot,
                )),
            );
            self.scene_sent = true;
        }
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

        let ubo = UniformBufferObject {
            model,
            view,
            proj,
        };

        copy_uniform_to_memory(&ubo, memory_target_ptr);
    }

    fn uniform_buffer_size(&self) -> usize {
        std::mem::size_of::<UniformBufferObject>()
    }
}
//////////////////////// Fns ///////////////////////
fn main() {
    env_logger::init();
    info!("Hello demo");
    MainLoopBuilder::new()
        .with_window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .expect("Failed to create window")
        .run(Demo::default());
}
