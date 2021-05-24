use std::path::PathBuf;

pub enum RenderCommand {
    DrawObject{
        texture_file: PathBuf,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
    },
}

#[derive(Default)]
pub struct RenderCommands {
    pub request_redraw: bool,
    pub command_queue: Vec<RenderCommand>,
}

impl RenderCommands {
    pub fn draw_textured_model(
        &mut self,
        texture_file: PathBuf,
        model_file: PathBuf,
        vertex_shader_file: PathBuf,
        fragment_shader_file: PathBuf,
    ) {
        self.command_queue.push(RenderCommand::DrawObject {
            texture_file,
            model_file,
            vertex_shader_file,
            fragment_shader_file,
        });
    }
}
