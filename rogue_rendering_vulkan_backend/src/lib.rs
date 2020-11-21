extern crate nalgebra_glm as glm;
extern crate rustylog;
extern crate memoffset;

pub mod devices;
pub mod drawing;
pub mod graphics_pipeline;
pub mod presentation;
pub mod util;
pub mod vertex_buffers;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
