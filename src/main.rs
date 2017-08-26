#[macro_use]
extern crate glium;
extern crate nalgebra;
extern crate image;

mod gui;
mod event;
mod asset_storage;

use std::cell::Cell;
use std::cmp::{max, min};

use gui::{View, Bbox, Point, SpaceDivBuilder, DivUnit, DivDirection, DivAlignment};
use gui::window::WindowBuilder;
use gui::renderer::Renderer;
use gui::widgets;

use asset_storage::{TextureId, AssetStorage};

fn main() {
    let running = Cell::new(true);

    let main_window = WindowBuilder::new()
        .with_dimensions(1024, 720)
        .with_title("Chorus Studio")
        .build()
        .unwrap();
    main_window.on_close().add_handler(|| running.set(false));

    let mut storage = AssetStorage::new();
    let image = storage.load_image_file("testimg.png").unwrap();
    let image = storage
        .create_texture(image, main_window.display())
        .unwrap();

    let mut renderer = Renderer::new(main_window.display(), &storage);

    *main_window.view() = main_screen(image);

    while running.get() {
        main_window.update(&mut renderer);
    }
}

fn main_screen(image: TextureId) -> View {
    let zero = Point::new(0, 0);
    let mut view = View::new(Bbox::with_size(zero, zero));

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
                    .build(),
            )
            .add_div(
                SpaceDivBuilder::new()
                    .width(DivUnit::Relative(1.0))
                    .height(DivUnit::Pixels(45))
                    .build(),
            )
            .build(),
    );

    view
}
