use std::cell::{RefCell, RefMut, Ref};
use std::default::Default;

use glium;
use glium::glutin;

use gui::{Point, Size, Bbox, View};
use gui::renderer::{Renderer, Painting};

use event::Event;

#[derive(Debug)]
pub enum WindowCreationError {
    GliumCreationError(glium::backend::glutin::DisplayCreationError),
}

pub struct WindowBuilder<'a> {
    window_builder: glutin::WindowBuilder,
    context_builder: glutin::ContextBuilder<'a>,
}

impl<'a> WindowBuilder<'a> {
    pub fn new() -> Self {
        Self {
            .. Default::default()
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.window_builder = self.window_builder.with_title(title);
        self
    }

    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.window_builder = self.window_builder.with_dimensions(width, height);
        self
    }

    pub fn with_multisampling(mut self, level: u16) -> Self {
        self.context_builder = self.context_builder.with_multisampling(level);
        self
    }

    pub fn build<'b>(self) -> Result<Window<'b>, WindowCreationError> {
        let events_loop = glutin::EventsLoop::new();
        let display = glium::Display::new(self.window_builder, self.context_builder, &events_loop).map_err(|e| WindowCreationError::GliumCreationError(e))?;
        Ok(Window::new(display, events_loop))
    }

    // pub fn build_shared<'b>(mut self, shared_from: &'a Window) -> Result<Window<'b>, WindowCreationError> {
    //     let win = shared_from.display.gl_window();
    //     self.context_builder = self.context_builder.with_shared_lists(win.context());
    //     self.build()
    // }
}

impl<'a> Default for WindowBuilder<'a> {
    fn default() -> Self {
        let builder = Self {
            window_builder: glutin::WindowBuilder::new(),
            context_builder: glutin::ContextBuilder::new()
                .with_gl_profile(glutin::GlProfile::Core)
                .with_gl(glutin::GlRequest::Latest)
                .with_vsync(true),
        };

        builder.with_dimensions(320, 240).with_multisampling(4)
    }
}

pub struct Window<'a> {
    display: glium::Display,
    events_loop: RefCell<glutin::EventsLoop>,
    view: RefCell<View>,
    on_close: RefCell<Event<'a>>,
}

impl<'a> Window<'a> {
    pub fn new(display: glium::Display, events_loop: glutin::EventsLoop) -> Self {
        let (w, h) = display.gl_window().get_inner_size_pixels().unwrap();
        let bbox = Bbox::with_size(Point::new(0, 0), Size::new(w as i32, h as i32));

        Self {
            display,
            events_loop: RefCell::new(events_loop),
            view: RefCell::new(View::new(bbox)),
            on_close: RefCell::new(Event::new()),
        }
    }

    pub fn on_close(&self) -> RefMut<Event<'a>> {
        self.on_close.borrow_mut()
    }

    pub fn update(&self, renderer: &mut Renderer) {
        self.events_loop.borrow_mut().poll_events(|evt| {
            use self::glutin::WindowEvent::*;

            match evt {
                glutin::Event::WindowEvent{event, ..} => match event {
                    Closed => self.on_close.borrow().invoke(),
                    Resized(w, h) => {
                        self.view.borrow_mut().set_bbox(Bbox::with_size(Point::new(0, 0), Point::new(w as i32, h as i32)));
                    }
                    _ => {},
                },
                _ => {},
            }
        });

        let mut painting = Painting::new(&self.display);
        self.view.borrow().draw(&mut painting);
        renderer.do_painting(painting);

        self.display.swap_buffers().unwrap();
    }

    pub fn display(&self) -> &glium::Display {
        &self.display
    }

    pub fn window(&self) -> Ref<glutin::GlWindow> {
        self.display.gl_window()
    }

    pub fn view(&self) -> RefMut<View> {
        self.view.borrow_mut()
    }
}