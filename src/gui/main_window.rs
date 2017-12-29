use gl;
use glutin;
use glutin::GlContext;
use nanovg;

use std::time::Instant;

use gui;

const INITIAL_WINDOW_SIZE: (u32, u32) = (1280, 720);
const RED: gui::Color = gui::Color {
    r: 0xff as f32 / 255.0,
    g: 0xca as f32 / 255.0,
    b: 0x77 as f32 / 255.0,
    a: 1.0,
};
//const BLUE: gui::Color = gui::Color {
//    r: 0x2e as f32 / 255.0,
//    g: 0x50 as f32 / 255.0,
//    b: 0x77 as f32 / 255.0,
//    a: 1.0,
//};
const BLUE: gui::Color = gui::Color {
    r: 112 as f32 / 255.0,
    g: 48 as f32 / 255.0,
    b: 160 as f32 / 255.0,
    a: 1.0,
};


/// Chorus Studio's main window.
pub struct MainWindow {
    events_loop: glutin::EventsLoop,
    gl_window: glutin::GlWindow,
    pub was_closed: bool,
    start_time: Instant,
}

impl MainWindow {
    pub fn new() -> Result<Self, CreateMainWindowError> {
        let events_loop = glutin::EventsLoop::new();
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(INITIAL_WINDOW_SIZE.0, INITIAL_WINDOW_SIZE.1)
            .with_title("Chorus Studio");
        let context_builder = glutin::ContextBuilder::new()
            .with_gl(glutin::GlRequest::Latest)
            .with_gl_profile(glutin::GlProfile::Core)
            .with_multisampling(4)
            .with_vsync(true);
        let gl_window = glutin::GlWindow::new(window_builder, context_builder, &events_loop)?;

        unsafe {
            gl_window.make_current()?;
            gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
            gl::ClearColor(0.2, 0.4, 0.8, 1.0);
        }

        Ok(MainWindow {
            events_loop,
            gl_window,
            was_closed: false,
            start_time: Instant::now(),
        })
    }

    /// Update and draw the window for one frame.
    pub fn update_draw(&mut self, view: &mut gui::View, context: &nanovg::Context) {
        let mut was_closed = self.was_closed;
        let gl_window = &self.gl_window;
        self.events_loop.poll_events(|event| match event {
           glutin::Event::WindowEvent { event, .. } => match event {
               glutin::WindowEvent::Closed => was_closed = true,
               glutin::WindowEvent::Resized(w, h) => gl_window.resize(w, h),
               _ => {}
           },
            _ => {}
        });
        self.was_closed = was_closed;

        let (w, h) = self.gl_window.get_inner_size().unwrap_or(INITIAL_WINDOW_SIZE);
        let (w, h) = (w as i32, h as i32);

        unsafe {
            let elapsed = self.start_time.elapsed();
            let elapsed = (elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 * 1e-9) as f32;
            let color = nanovg::Color::lerp(RED.into(), BLUE.into(), (elapsed / 2.0).cos() * 0.5 + 0.5);
            gl::ClearColor(color.red(), color.green(), color.blue(), color.alpha());
            gl::Viewport(0, 0, w, h);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        view.set_bbox(gui::Bbox::with_size(gui::Point::new(0, 0), gui::Point::new(w, h)));
        context.frame((w, h), self.gl_window.hidpi_factor(), |frame| {
            view.draw(&frame);
        });

        let _ = self.gl_window.swap_buffers();
    }
}

#[derive(Debug)]
pub enum CreateMainWindowError {
    CreationError(glutin::CreationError),
    ContextError(glutin::ContextError),
}

impl From<glutin::CreationError> for CreateMainWindowError {
    fn from(e: glutin::CreationError) -> Self {
        CreateMainWindowError::CreationError(e)
    }
}

impl From<glutin::ContextError> for CreateMainWindowError {
    fn from(e: glutin::ContextError) -> Self {
        CreateMainWindowError::ContextError(e)
    }
}