extern crate bincode;
extern crate gl;
extern crate nanovg;
extern crate proto;
extern crate sdl2;

mod render;
mod ui;

use std::cell::RefCell;
use std::net;
use std::sync;
use std::thread;
use std::time;

const SERVER_IP: &str = "127.0.0.1:4450";

enum MainThreadMsg {
    Shutdown,
    Command(proto::Command),
}

enum NetThreadMsg {
    Connected,
    Response(proto::Response),
}

fn main() {
    // SDL
    let sdl = sdl2::init().expect("SDL2 init");
    let video = sdl.video().expect("SDL2 video");
    video.gl_set_swap_interval(sdl2::video::SwapInterval::VSync);
    let mut events = sdl.event_pump().expect("SDL2 event pump");
    let mut window = video
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

    let load_task = RefCell::new("Connecting to server...".to_owned());
    let cur_users = RefCell::new(Vec::new());
    let mut cur_view: Box<dyn ui::View> = Box::new(ui::views::MainLoadingView {
        cur_load_task: &load_task,
    });

    // Networking
    let (main_tx, main_rx) = sync::mpsc::channel();
    let (server_tx, server_rx) = sync::mpsc::channel();
    let network_thread = thread::spawn(move || -> Result<(), ()> {
        let mut stream = net::TcpStream::connect(SERVER_IP).map_err(|_| ())?;
        server_tx.send(NetThreadMsg::Connected).map_err(|_| ())?;
        stream
            .set_read_timeout(Some(time::Duration::from_millis(500)))
            .map_err(|_| ())?;
        loop {
            if let Ok(msg) = main_rx.try_recv() {
                match msg {
                    MainThreadMsg::Shutdown => return Ok(()),
                    MainThreadMsg::Command(cmd) => {
                        bincode::serialize_into(&mut stream, &cmd).map_err(|_| ())?;
                    }
                }
            }

            let resp = bincode::deserialize_from::<_, proto::Response>(&stream);
            if let Ok(resp) = resp {
                server_tx
                    .send(NetThreadMsg::Response(resp))
                    .map_err(|_| ())?;
            }
        }
    });

    // Fonts
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
    {
        let render_ctx = render::RenderContext::new(&window, &nvg, fonts);

        'main: loop {
            use sdl2::event::Event::*;
            for e in events.poll_iter() {
                match e {
                    Quit { .. } => break 'main,
                    _ => {}
                }
            }

            for msg in server_rx.try_iter() {
                match msg {
                    NetThreadMsg::Connected => {
                        cur_view = Box::new(ui::views::MainView {
                            user_list: &cur_users,
                        });
                        let _ = main_tx.send(MainThreadMsg::Command(proto::Command::ListUsers));
                    }
                    NetThreadMsg::Response(res) => match res {
                        proto::Response::UserList(users) => {
                            cur_users.replace(users);
                        }
                    },
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
        }
    }

    window.hide();

    if let Ok(..) = main_tx.send(MainThreadMsg::Shutdown) {
        let _ = network_thread.join();
    }
}
