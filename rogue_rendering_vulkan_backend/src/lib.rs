extern crate rustylog;

pub mod devices;
pub mod graphics_pipeline;
pub mod presentation;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
