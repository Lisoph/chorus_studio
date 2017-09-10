use gui::Bbox;
use gui::renderer::{Color, Painting, TexturedRectangle};
use gui::renderer::text::Text;
use std::borrow::Cow;

use unicode_normalization::UnicodeNormalization;

use asset_storage::{TextureId, FontId};

pub trait Widget {
    fn draw<'a>(&'a self, _bbox: Bbox, _painting: &mut Painting) {}
    fn update(&mut self, _bbox: Bbox) {}
    // fn handle_window_event(&mut self, event: &glutin::WindowEvent);
}

pub struct Image {
    texture: TextureId,
}

impl Image {
    pub fn new(texture: TextureId) -> Self {
        Self { texture }
    }
}

impl Widget for Image {
    fn draw(&self, bbox: Bbox, painting: &mut Painting) {
        painting.draw(TexturedRectangle(bbox, self.texture));
    }
}

pub struct Label<'a> {
    font: FontId,
    color: Color,
    size: f32,
    text: Cow<'a, str>,
}

impl<'a> Label<'a> {
    pub fn new(font: FontId, color: Color, size: f32, text: Cow<'a, str>) -> Self {
        Self {
            font,
            color,
            size,
            text,
        }
    }
}

impl<'a> Widget for Label<'a> {
    fn draw(&self, bbox: Bbox, painting: &mut Painting) {
        let text: String = self.text.nfc().collect();
        painting.draw(Text::new(
            bbox,
            self.font,
            self.color,
            ::rusttype::Scale::uniform(self.size),
            text,
        ));
    }
}
