#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate image;
extern crate rusttype;
extern crate unicode_normalization;
extern crate unicode_segmentation;

mod gui;
mod event;
mod asset_storage;

use std::cell::{Cell, RefCell};
use std::borrow::Cow;
use std::rc::Rc;

use gui::{View, SpaceDivBuilder, DivUnit, DivAlignment};
use gui::window::WindowBuilder;
use gui::renderer::{Color, Renderer};
use gui::widgets;

use asset_storage::{TextureId, FontId, AssetStorage};

fn main() {
    let running = Cell::new(true);

    let main_window = WindowBuilder::new()
        .with_dimensions(1024, 720)
        .with_title("Chorus Studio")
        .build()
        .unwrap();
    main_window.on_close().add_handler(|| running.set(false));

    let mut storage = Rc::new(RefCell::new(AssetStorage::new()));
    let image = storage.borrow_mut().load_image_file("testimg.png").unwrap();
    let image = storage.borrow_mut()
        .create_texture(image, main_window.display())
        .unwrap();
    let font = storage.borrow_mut().load_font_file("testfont.ttf").unwrap();

    let mut renderer = Renderer::new(&main_window, Rc::clone(&storage));

    *main_window.view() = main_screen(image, font);

    while running.get() {
        main_window.update(&mut renderer);
    }
}

fn main_screen(image: TextureId, font: FontId) -> View {
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
                        Cow::Borrowed("Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.\n\nこのテキストはGoogle翻訳で翻訳されているため、おそらくあまり意味をなさないでしょう。\n\n这个文本可能没有什么意义，因为它是用Google翻译翻译的。"),
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
                        Cow::Borrowed(
                            "Hi! I'm a multi-\nline Text, containg some magnificent words.",
                        ),
                    )))
                    .build(),
            )
            .build(),
    );

    view
}
