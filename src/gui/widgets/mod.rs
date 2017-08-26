use gui::Bbox;
use gui::renderer::{Color, Painting, TexturedRectangle};

use asset_storage::{AssetStorage, TextureId};

pub trait Widget {
    fn draw(&self, bbox: Bbox, painting: &mut Painting);
    fn update(&mut self, bbox: Bbox);
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

    fn update(&mut self, _bbox: Bbox) {}
}
