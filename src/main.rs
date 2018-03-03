#![feature(duration_extras)]

extern crate gl;
extern crate glutin;
extern crate indextree;
extern crate nalgebra;
extern crate nanovg;
extern crate unicode_normalization;

mod gui;
mod event;

use std::cell::Cell;
use std::time::Instant;

use indextree as it;

use gui::{Color, Point, View};
use gui::div;
use gui::main_window::MainWindow;
use gui::widget;

fn main() {
    let running = Cell::new(true);
    let mut main_window = MainWindow::new().expect("Failed to create window!");
    main_window.on_close.add_handler(|| running.set(false));

    let nvg = nanovg::ContextBuilder::new()
        .stencil_strokes()
        .build()
        .expect("Failed to create NanoVG context!");
    let image = nanovg::Image::new(&nvg)
        .build_from_file("assets/testimg.png")
        .expect("Failed to load image!");
    let font_arial = nanovg::Font::from_file(&nvg, "testfont", "assets/arial.ttf")
        .expect("Failed to load font!");
    let font_moderno =
        nanovg::Font::from_file(&nvg, "modern", "assets/moderno.ttf").expect("Modernism.ttf");

    let mut main_screen = MainScreen::new(font_moderno, font_arial);
    let time_start = Instant::now();

    while running.get() {
        let delta = time_start.elapsed();
        let delta = (delta.as_secs() * 1000 + delta.subsec_millis() as u64) as f32 / 1000.0;
        let delta = delta.sin() * 0.5 + 0.5;
        main_screen
            .view
            .space_div(main_screen.chat)
            .scroll
            .set(Point::new((delta * 40.0) as i32, (delta * 200.0) as i32));

        main_screen.update();
        main_window.update_draw(&mut main_screen.view, &nvg);
    }
}

struct MainScreen<'a> {
    view: View<'a>,
    header: it::NodeId,
    header_title: it::NodeId,
    body: it::NodeId,
    chat: it::NodeId,
    ticks: u64,
    messages: Vec<it::NodeId>,
    chat_font: nanovg::Font<'a>,
}

impl<'a> MainScreen<'a> {
    fn new(title_font: nanovg::Font<'a>, chat_font: nanovg::Font<'a>) -> Self {
        let mut view = View::without_bbox();
        let header = view.add_div(
            None,
            div::SpaceDivBuilder::new()
                .width(div::Unit::Relative(1.0))
                .height(div::Unit::Pixels(40))
                .background_color(Color::rgba(0.0, 1.0, 0.0, 0.2))
                .build(),
        );

        let header_title = view.add_div(
            Some(header),
            div::SpaceDivBuilder::new()
                .width(div::Unit::Relative(0.25))
                .height(div::Unit::Relative(1.0))
                .min_width(div::Unit::Pixels(250))
                .widget(Box::new(widget::Label::new(
                    title_font,
                    Color::white(),
                    32.0,
                    "Chorus Studio",
                )))
                .build(),
        );

        let body = view.add_div(
            None,
            div::SpaceDivBuilder::new()
                .width(div::Unit::Relative(1.0))
                .height(div::Unit::Calc(Box::new(|data| data.remaining - 100)))
                .horizontal()
                .build(),
        );

        let chat = view.add_div(
            Some(body),
            div::SpaceDivBuilder::new()
                .width(div::Unit::Relative(0.25))
                .height(div::Unit::Relative(1.0))
                .min_width(div::Unit::Pixels(100))
                .vertical()
                .vert_align(div::Alignment::Min)
                .hori_overflow(div::Overflow::Scroll)
                .vert_overflow(div::Overflow::Scroll)
                .background_color(Color::rgba(0.4, 0.2, 0.0, 1.0))
                .build(),
        );

        Self {
            view,
            header,
            header_title,
            body,
            chat,
            ticks: 0,
            messages: Vec::new(),
            chat_font,
        }
    }

    fn update(&mut self) {
        self.ticks += 1;
        if self.ticks % 25 == 0 {
            self.view.add_div(
                Some(self.chat),
                div::SpaceDivBuilder::new()
                    .width(div::Unit::Relative(1.5)) // Too wide on purpose
                    .height(div::Unit::Pixels(32))
                    .widget(Box::new(widget::Label::new(
                        self.chat_font,
                        Color::white(),
                        20.0,
                        "fake chat message",
                    )))
                    .background_color(Color::rgba(0.7, 0.2, 0.1, 1.0))
                    .build(),
            );
        }
    }
}
