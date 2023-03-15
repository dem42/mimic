use mimic_common::uniforms::UniformSpec;
use std::{path::PathBuf, rc::Rc};
//////////////////////// Enums ///////////////////////
pub enum RenderCommand {
    DrawObject {
        texture_file: Rc<PathBuf>,
        model_file: Rc<PathBuf>,
        vertex_shader_file: Rc<PathBuf>,
        fragment_shader_file: Rc<PathBuf>,
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
        texture_file: &Rc<PathBuf>,
        model_file: &Rc<PathBuf>,
        vertex_shader_file: &Rc<PathBuf>,
        fragment_shader_file: &Rc<PathBuf>,
        uniform_spec: Box<dyn UniformSpec>,
    ) {
        self.command_queue.push(RenderCommand::DrawObject {
            texture_file: Rc::clone(texture_file),
            model_file: Rc::clone(model_file),
            vertex_shader_file: Rc::clone(vertex_shader_file),
            fragment_shader_file: Rc::clone(fragment_shader_file),
            uniform_spec,
        });
    }
}
