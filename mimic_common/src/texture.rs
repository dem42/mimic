use image::GenericImageView;
use std::{path::PathBuf, rc::Rc};

use crate::result::Result;
//////////////////////// Traits ///////////////////////
pub trait TextureSource {
    fn get_pixels(&self) -> &Vec<u8>;
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
    fn get_image_size(&self) -> u32;
}
//////////////////////// Structs ///////////////////////
#[derive(Default)]
pub struct FilesystemTextureSource {
    pub path: Rc<PathBuf>,
    width: u32,
    height: u32,
    image_size: u32,
    pixels: Vec<u8>,
}
//////////////////////// Impls ///////////////////////
impl FilesystemTextureSource {
    pub fn new(path: &Rc<PathBuf>) -> Result<Self> {
        let image = image::open(path.as_path())?;

        let (width, height) = image.dimensions();
        let image_size = width * height * 4;

        let rgba_image = image.into_rgba8();
        let pixels: &Vec<u8> = rgba_image.as_raw();

        Ok(FilesystemTextureSource {
            path: Rc::clone(path),
            width,
            height,
            image_size,
            pixels: pixels.clone(),
        })
    }
}

impl TextureSource for FilesystemTextureSource {
    fn get_pixels(&self) -> &Vec<u8> {
        &self.pixels
    }
    fn get_width(&self) -> u32 {
        self.width
    }
    fn get_height(&self) -> u32 {
        self.height
    }
    fn get_image_size(&self) -> u32 {
        self.image_size
    }
}
