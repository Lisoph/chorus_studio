extern crate bincode;
extern crate gl;
extern crate nanovg;
extern crate proto;
extern crate sdl2;

mod render;
mod ui;

use std::net;

const SERVER_IP: &str = "127.0.0.1:4450";

fn main() {
    let sdl = sdl2::init().expect("SDL2 init");
    let video = sdl.video().expect("SDL2 video");
    video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
    let mut events = sdl.event_pump().expect("SDL2 event pump");
    let window = video
        .window("Chorus Studio", 1280, 720)
        .opengl()
        .position_centered()
        .resizable()
        .build()
        .expect("SDL2 window");
    let glctx = window.gl_create_context().expect("OpenGL context");
    window.gl_make_current(&glctx).expect("OpenGL make current");
    gl::load_with(|s| video.gl_get_proc_address(s) as *const _);
    let nvg = nanovg::ContextBuilder::new()
        .stencil_strokes()
        .build()
        .expect("NanoVG context");

    let mut cur_view: Box<dyn ui::View> = Box::new(ui::views::MainLoadingView {
        cur_load_task: "Connecting to server...".to_owned(),
    });

    let mut stream = net::TcpStream::connect(SERVER_IP).expect("Server connect");
    bincode::serialize_into(&mut stream, &proto::Command::ListUsers).expect("serialized");
    let _resp: proto::Response = bincode::deserialize_from(&stream).expect("Read response");

    use render::Fonts;
    let mut fonts: [nanovg::Font; Fonts::NumFonts as usize] = unsafe { std::mem::uninitialized() };
    fonts[Fonts::Inter as usize] =
        nanovg::Font::from_file(&nvg, "Inter UI", "assets/Inter-UI-Regular.ttf")
            .expect("Font Inter");
    fonts[Fonts::Vga8 as usize] =
        nanovg::Font::from_file(&nvg, "PxPlus IBM VGA8", "assets/PxPlus_IBM_VGA8.ttf")
            .expect("Font VGA8");
    fonts[Fonts::Moderno as usize] =
        nanovg::Font::from_file(&nvg, "Moderno", "assets/moderno.ttf").expect("Font Moderno");
    let render_ctx = render::RenderContext::new(&window, &nvg, fonts);

    let mut frames = 0usize;

    'main: loop {
        use sdl2::event::Event::*;
        for e in events.poll_iter() {
            match e {
                Quit { .. } => break 'main,
                _ => {}
            }
        }

        unsafe {
            let (w, h) = window.size();
            gl::Viewport(0, 0, w as i32, h as i32);
            gl::ClearColor(0.2, 0.4, 0.8, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }

        cur_view.present(&render_ctx);
        window.gl_swap_window();

        match frames {
            1000 => {
                let loading =
                    unsafe { &mut *(&mut *cur_view as *mut _ as *mut ui::views::MainLoadingView) };
                loading.cur_load_task = "Almost done...".to_owned();
            }
            1200 => cur_view = Box::new(ui::views::MainView {}),
            _ => {}
        }

        frames += 1;
    }
}
