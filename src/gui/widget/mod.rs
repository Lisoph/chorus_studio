use gui::{Bbox, Color};
use nanovg;

pub trait Widget {
    fn draw(&self, bbox: Bbox, clip: Bbox, frame: &nanovg::Frame);
    fn update(&mut self, bbox: Bbox);
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
    fn draw(&self, bbox: Bbox, clip: Bbox, frame: &nanovg::Frame) {
        frame.path(|path| {
            let size = bbox.size();
            let origin = (bbox.min.x as f32, bbox.min.y as f32);
            let size = (size.x as f32, size.y as f32);
            path.rect(origin, size);
            path.fill(nanovg::FillStyle {
                coloring_style: nanovg::ColoringStyle::Paint(nanovg::Paint::with_image_pattern(frame.context(), &self.image, origin, size, 0.0, 1.0)),
                ..Default::default()
            })
        }, nanovg::PathOptions {
            scissor: Some(nanovg::Scissor::Rect {
                x: clip.min.x as f32,
                y: clip.min.y as f32,
                width: clip.size().x as f32,
                height: clip.size().x as f32,
            }),
            .. Default::default()
        });
    }

    fn update(&mut self, bbox: Bbox) {}
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
    fn draw(&self, bbox: Bbox, clip: Bbox, frame: &nanovg::Frame) {
        let clip = bbox.overlapping(clip).unwrap_or(clip);
        frame.context().text_box(self.font, (bbox.min.x as f32, bbox.min.y as f32),
            self.text, nanovg::TextOptions {
                size: self.size,
                color: self.color.into(),
                line_max_width: bbox.size().x as f32,
                scissor: Some(nanovg::Scissor::Rect {
                    x: clip.min.x as f32,
                    y: clip.min.y as f32,
                    width: clip.size().x as f32,
                    height: clip.size().y as f32,
                }),
                ..Default::default()
            });
    }

    fn update(&mut self, bbox: Bbox) {}
}
