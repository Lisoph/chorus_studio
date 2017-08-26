use std::collections::HashMap;
use std::path::Path;
use std::cell::RefCell;
use std::ops::RangeFrom;

use image;
use image::DynamicImage;

use glium;
use glium::backend::Facade;
use glium::texture::{CompressedSrgbTexture2d, RawImage2d};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ImageId(u32);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct TextureId(u32);

pub struct AssetStorage {
    image_id_generator: RefCell<RangeFrom<u32>>,
    texture_id_generator: RefCell<RangeFrom<u32>>,
    images: HashMap<ImageId, DynamicImage>,
    textures: HashMap<TextureId, CompressedSrgbTexture2d>,
}

#[derive(Debug)]
pub enum ImageLoadError {
    ImageError(image::ImageError),
}

impl From<image::ImageError> for ImageLoadError {
    fn from(err: image::ImageError) -> Self {
        ImageLoadError::ImageError(err)
    }
}

pub type ImageLoadResult = Result<ImageId, ImageLoadError>;

#[derive(Debug)]
pub enum CreateTextureError {
    InvalidImageId(ImageId),
    TextureCreationError(glium::texture::TextureCreationError),
}

impl From<glium::texture::TextureCreationError> for CreateTextureError {
    fn from(err: glium::texture::TextureCreationError) -> Self {
        CreateTextureError::TextureCreationError(err)
    }
}

impl AssetStorage {
    pub fn new() -> Self {
        Self {
            image_id_generator: RefCell::new(0..),
            texture_id_generator: RefCell::new(0..),
            images: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    fn gen_image_id(&self) -> ImageId {
        let mut gen = self.image_id_generator.borrow_mut();
        let next = gen.next().unwrap();
        ImageId(next)
    }

    fn gen_texture_id(&self) -> TextureId {
        let mut gen = self.texture_id_generator.borrow_mut();
        let next = gen.next().unwrap();
        TextureId(next)
    }

    pub fn load_image_file<P: AsRef<Path>>(&mut self, path: P) -> ImageLoadResult {
        let img = image::open(path)?;
        let id = self.gen_image_id();
        let _ = self.images.insert(id, img);
        Ok(id)
    }

    pub fn image(&self, id: ImageId) -> Option<&DynamicImage> {
        self.images.get(&id)
    }

    pub fn create_texture<F: Facade>(
        &mut self,
        image: ImageId,
        facade: &F,
    ) -> Result<TextureId, CreateTextureError> {
        let raw = {
            let image = match self.image(image) {
                Some(image) => image,
                None => return Err(CreateTextureError::InvalidImageId(image)),
            };

            let image = image.to_rgba();
            let dimensions = image.dimensions();
            let raw = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), dimensions);
            raw
        };

        let texture = CompressedSrgbTexture2d::new(facade, raw)?;

        let id = self.gen_texture_id();
        let _ = self.textures.insert(id, texture);
        Ok(id)
    }

    pub fn texture(&self, id: TextureId) -> Option<&CompressedSrgbTexture2d> {
        self.textures.get(&id)
    }
}
