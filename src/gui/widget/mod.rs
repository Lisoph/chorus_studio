use gui::{Bbox, Color, div};
use nanovg;

pub trait Widget {
    /// Draw the widget in its current state on the screen.
    /// `bbox` is the size the div ideally would like to have. However, since divs can overflow
    /// and be clipped, the actual bounding box in which the widget can be drawn might be smaller
    /// than the div's `bbox`. The parameter `clip` specifies this actual bounding box which
    /// the widget must not exceed.
    fn draw(&self, bbox: Bbox, clip: Bbox, frame: &nanovg::Frame);
    /// Update the widgets internal state. Get's called before every draw invocation.
    fn update(&mut self);
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
        frame.path(
            |path| {
                let size = bbox.size();
                let origin = (bbox.min.x as f32, bbox.min.y as f32);
                let size = (size.x as f32, size.y as f32);
                path.rect(origin, size);
                path.fill(nanovg::FillStyle {
                    coloring_style: nanovg::ColoringStyle::Paint(
                        nanovg::Paint::with_image_pattern(
                            frame.context(),
                            &self.image,
                            origin,
                            size,
                            0.0,
                            1.0,
                        ),
                    ),
                    ..Default::default()
                })
            },
            nanovg::PathOptions {
                scissor: Some(clip.into()),
                ..Default::default()
            },
        );
    }

    fn update(&mut self) {}
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
        frame.context().text_box(
            self.font,
            (bbox.min.x as f32, bbox.min.y as f32),
            self.text,
            nanovg::TextOptions {
                size: self.size,
                color: self.color.into(),
                line_max_width: bbox.size().x as f32,
                scissor: Some(clip.into()),
                ..Default::default()
            },
        );
    }

    fn update(&mut self) {
        
    }
}
