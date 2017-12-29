use gui::{Bbox, Color};
use nanovg;

pub trait Widget {
    fn draw(&self, _bbox: Bbox, _frame: &nanovg::Frame) {}
    fn update(&mut self, _bbox: Bbox) {}
    // fn handle_window_event(&mut self, event: &glutin::WindowEvent);
}

pub struct Image<'a> {
    image: nanovg::Image<'a>,
}

impl<'a> Image<'a> {
    pub fn new(image: nanovg::Image<'a>) -> Self {
        Self { image }
    }
}

impl<'a> Widget for Image<'a> {
    fn draw(&self, bbox: Bbox, frame: &nanovg::Frame) {
        frame.path(|path| {
            let size = bbox.size();
            let origin = (bbox.min.x as f32, bbox.min.y as f32);
            let size = (size.x as f32, size.y as f32);
            path.rect(origin, size);
            path.fill(nanovg::FillStyle {
                coloring_style: nanovg::ColoringStyle::Paint(nanovg::Paint::with_image_pattern(frame.context(), &self.image, origin, size, 0.0, 1.0)),
                .. Default::default()
            })
        }, Default::default());
    }
}

pub struct Label<'a> {
    font: nanovg::Font<'a>,
    color: Color,
    size: f32,
    text: &'a str,
}

impl<'a> Label<'a> {
    pub fn new(font: nanovg::Font<'a>, color: Color, size: f32, text: &'a str) -> Self {
        Self {
            font,
            color,
            size,
            text,
        }
    }
}

impl<'a> Widget for Label<'a> {
    fn draw(&self, bbox: Bbox, frame: &nanovg::Frame) {
        use unicode_normalization::UnicodeNormalization;

        let text: String = self.text.nfc().collect();
        let bbox_size = bbox.size();
        frame.context().text_box(self.font, (bbox.min.x as f32, bbox.min.y as f32),
                                 text, nanovg::TextOptions {
            size: self.size,
            color: self.color.into(),
            line_max_width: bbox_size.x as f32,
            scissor: Some(nanovg::Scissor::Rect {
                x: bbox.min.x as f32,
                y: bbox.min.y as f32,
                width: bbox_size.x as f32,
                height: bbox_size.y as f32,
            }),
            .. Default::default()
        });
    }
}
