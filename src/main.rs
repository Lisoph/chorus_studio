extern crate gl;
extern crate glutin;
extern crate nanovg;
extern crate nalgebra;
extern crate unicode_normalization;

mod gui;
mod event;

use std::cell::Cell;

use gui::{View, SpaceDivBuilder, DivUnit, DivAlignment, Color};
use gui::main_window::MainWindow;
use gui::widgets;

fn main() {
    let running = Cell::new(true);
    let mut main_window = MainWindow::new().expect("Failed to create window!");
    main_window.on_close.add_handler(|| running.set(false));

    let nvg = nanovg::ContextBuilder::new().stencil_strokes().build().expect("Failed to create NanoVG context!");
    let image = nanovg::Image::new(&nvg).build_from_file("testimg.png").expect("Failed to load image!");
    let font = nanovg::Font::from_file(&nvg, "testfont", "testfont.ttf").expect("Failed to load font!");

    let mut main_screen = main_screen(image, font);

    while running.get() {
        main_window.update_draw(&mut main_screen, &nvg);
    }
}

fn main_screen<'a>(image: nanovg::Image<'a>, font: nanovg::Font<'a>) -> View<'a> {
    let mut view = View::without_bbox();

    // Header (logo)
    view.add_div(
        SpaceDivBuilder::new()
            .width(DivUnit::Relative(1.0))
            .height(DivUnit::Pixels(150))
            .hori_align(DivAlignment::Center)
            .add_div(
                SpaceDivBuilder::new()
                    .width(DivUnit::Relative(1.0))
                    .min_width(DivUnit::Pixels(640))
                    .max_width(DivUnit::Pixels(1100))
                    .height(DivUnit::Pixels(150))
                    .widget(Box::new(widgets::Image::new(image)))
                    .build(),
            )
            .build(),
    );

    view.add_div(
        SpaceDivBuilder::new()
            .horizontal()
            .width(DivUnit::Relative(1.0))
            .height(DivUnit::Relative(0.5))
            .vert_align(DivAlignment::Center)
            .add_div(
                SpaceDivBuilder::new()
                    .width(DivUnit::Relative(0.5))
                    .height(DivUnit::Relative(1.0))
                    .widget(Box::new(widgets::Label::new(
                        font,
                        Color::white(),
                        24.0,
                        "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.\n\nこのテキストはGoogle翻訳で翻訳されているため、おそらくあまり意味をなさないでしょう。\n\n这个文本可能没有什么意义，因为它是用Google翻译翻译的。",
                    )))
                    .build(),
            )
            .add_div(
                SpaceDivBuilder::new()
                    .width(DivUnit::Relative(0.3))
                    .height(DivUnit::Pixels(45))
                    .widget(Box::new(widgets::Label::new(
                        font,
                        Color::red(),
                        24.0,
                        "Hi! I'm a multi-\nline Text and I'm overflowing. Oh noes!",
                    )))
                    .build(),
            )
            .build(),
    );

    view
}
