use mimic_common::{texture::TextureSource, uniforms::UniformSpec};
use std::path::PathBuf;
//////////////////////// Enums ///////////////////////
pub enum RenderCommand {
    DrawObject {
        texture_source: Box<dyn TextureSource>,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
        uniform_spec: Box<dyn UniformSpec>,
    },
}
//////////////////////// Structs ///////////////////////
#[derive(Default)]
pub struct RenderCommands {
    pub request_redraw: bool,
    pub command_queue: Vec<RenderCommand>,
}
//////////////////////// Impls ///////////////////////
impl RenderCommands {
    pub fn draw_textured_model(
        &mut self,
        texture_source: Box<dyn TextureSource>,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
        uniform_spec: Box<dyn UniformSpec>,
    ) {
        self.command_queue.push(RenderCommand::DrawObject {
            texture_source,
            model_file,
            vertex_shader_file,
            fragment_shader_file,
            uniform_spec,
        });
    }
}
