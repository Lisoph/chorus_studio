use std::collections::HashMap;
use std::path::Path;
use std::cell::RefCell;
use std::ops::RangeFrom;
use std::io::{Error as IoError, Read};
use std::fs::File;

use image;
use image::DynamicImage;

use glium;
use glium::backend::Facade;
use glium::texture::{CompressedSrgbTexture2d, RawImage2d};

use rusttype::{FontCollection, Font};

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct ImageId(u32);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct TextureId(u32);

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct FontId(u32);

impl FontId {
    pub fn font_id(&self) -> usize {
        self.0 as usize
    }
}

pub struct AssetStorage<'a> {
    image_id_generator: RefCell<RangeFrom<u32>>,
    texture_id_generator: RefCell<RangeFrom<u32>>,
    font_id_generator: RefCell<RangeFrom<u32>>,
    // TODO: Do we really need to a copy of all images in main memory?
    //       They're only used on the GPU, so what's the point of having them in memory?
    images: HashMap<ImageId, DynamicImage>,
    textures: HashMap<TextureId, CompressedSrgbTexture2d>,
    fonts: HashMap<FontId, Font<'a>>,
}

#[derive(Debug)]
pub enum ImageLoadError {
    ImageError(image::ImageError),
    IdGenError,
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
    IdGenError,
}

impl From<glium::texture::TextureCreationError> for CreateTextureError {
    fn from(err: glium::texture::TextureCreationError) -> Self {
        CreateTextureError::TextureCreationError(err)
    }
}

pub type CreateTextureResult = Result<TextureId, CreateTextureError>;

#[derive(Debug)]
pub enum LoadFontError {
    IoError(IoError),
    InvalidFont,
    IdGenError,
}

impl From<IoError> for LoadFontError {
    fn from(err: IoError) -> Self {
        LoadFontError::IoError(err)
    }
}

pub type LoadFontResult = Result<FontId, LoadFontError>;

impl<'a> AssetStorage<'a> {
    pub fn new() -> Self {
        Self {
            image_id_generator: RefCell::new(0..),
            texture_id_generator: RefCell::new(0..),
            font_id_generator: RefCell::new(0..),
            images: HashMap::new(),
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    fn gen_image_id(&self) -> ImageLoadResult {
        match self.image_id_generator.borrow_mut().next() {
            Some(id) => Ok(ImageId(id)),
            None => Err(ImageLoadError::IdGenError),
        }
    }

    fn gen_texture_id(&self) -> CreateTextureResult {
        match self.texture_id_generator.borrow_mut().next() {
            Some(id) => Ok(TextureId(id)),
            None => Err(CreateTextureError::IdGenError),
        }
    }

    fn gen_font_id(&self) -> LoadFontResult {
        match self.font_id_generator.borrow_mut().next() {
            Some(id) => Ok(FontId(id)),
            None => Err(LoadFontError::IdGenError),
        }
    }

    pub fn load_image_file<P: AsRef<Path>>(&mut self, path: P) -> ImageLoadResult {
        let id = self.gen_image_id()?;
        let _ = self.images.insert(id, image::open(path)?);
        Ok(id)
    }

    pub fn image(&'a self, id: ImageId) -> Option<&'a DynamicImage> {
        self.images.get(&id)
    }

    pub fn create_texture<F: Facade>(&mut self, image: ImageId, facade: &F) -> CreateTextureResult {
        let raw = {
            let image = match self.image(image) {
                Some(image) => image,
                None => return Err(CreateTextureError::InvalidImageId(image)),
            };

            let image = image.to_rgba();
            let dimensions = image.dimensions();
            let raw = RawImage2d::from_raw_rgba(image.into_raw(), dimensions);
            raw
        };

        let texture = CompressedSrgbTexture2d::new(facade, raw)?;

        let id = self.gen_texture_id()?;
        let _ = self.textures.insert(id, texture);
        Ok(id)
    }

    pub fn texture(&'a self, id: TextureId) -> Option<&'a CompressedSrgbTexture2d> {
        self.textures.get(&id)
    }

    pub fn load_font_file<P: AsRef<Path>>(&mut self, path: P) -> LoadFontResult {
        let mut file = File::open(path)?;
        let mut bytes = Vec::new();
        let _ = file.read_to_end(&mut bytes)?;
        let font = match FontCollection::from_bytes(bytes).into_font() {
            Some(font) => font,
            None => return Err(LoadFontError::InvalidFont),
        };

        let id = self.gen_font_id()?;
        let _ = self.fonts.insert(id, font);
        Ok(id)
    }

    pub fn font(&'a self, id: FontId) -> Option<&'a Font> {
        self.fonts.get(&id)
    }
}
