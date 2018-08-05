use nanovg::{Color, TextOptions};

use render::{Fonts, RenderContext};

pub struct MainLoadingView {
    pub cur_load_task: String,
}

impl super::View for MainLoadingView {
    fn present(&mut self, ctx: &RenderContext) {
        let (w, h) = ctx.size();
        ctx.frame(|f| {
            f.path(
                |p| {
                    let radius = 30.0;
                    p.circle((w / 2.0, h / 2.0), radius);
                    p.fill(Color::from_rgb(200, 100, 0), Default::default());
                },
                Default::default(),
            );
            // Chorus Studio
            {
                let size = 60.0;
                let text = "Chorus Studio";
                let font = ctx.font(Fonts::Moderno);
                let (_, bounds) = f.text_bounds(
                    font,
                    (0.0, 0.0),
                    text,
                    TextOptions {
                        size: size,
                        ..Default::default()
                    },
                );
                let (width, height) = (bounds.max_x, bounds.max_y);
                f.text(
                    font,
                    ((w - width) / 2.0, (h - height) / 8.0),
                    text,
                    TextOptions {
                        size: size,
                        color: Color::from_rgb(255, 255, 255),
                        ..Default::default()
                    },
                );
            }
            // loading
            {
                let size = 28.0;
                let font = ctx.font(Fonts::Vga8);
                let (_, bounds) = f.text_bounds(
                    font,
                    (0.0, 0.0),
                    &self.cur_load_task,
                    TextOptions {
                        size: size,
                        ..Default::default()
                    },
                );
                let (width, height) = (bounds.max_x, bounds.max_y);
                f.text(
                    font,
                    ((w - width) / 2.0, (h - height) / 4.0),
                    &self.cur_load_task,
                    TextOptions {
                        size: size,
                        color: Color::from_rgb(200, 200, 200),
                        ..Default::default()
                    },
                );
            }
        });
    }
}

pub struct MainView {}

impl super::View for MainView {
    fn present(&mut self, ctx: &RenderContext) {}
}
