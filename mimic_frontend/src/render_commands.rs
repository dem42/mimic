use mimic_common::uniforms::UniformMetadata;
use std::path::PathBuf;
//////////////////////// Enums ///////////////////////
pub enum RenderCommand {
    DrawObject {
        texture_file: PathBuf,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
        uniform_metadata: UniformMetadata,
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
        texture_file: PathBuf,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
        uniform_metadata: UniformMetadata,
    ) {
        self.command_queue.push(RenderCommand::DrawObject {
            texture_file,
            model_file,
            vertex_shader_file,
            fragment_shader_file,
            uniform_metadata,
        });
    }
}
