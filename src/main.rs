extern crate bincode;
extern crate glfw_ffi;
extern crate nanovg;
extern crate proto;
extern crate sha3;

mod gl;
mod render;
mod ui;

use glfw_ffi::*;

use std::cell::RefCell;
use std::net;
use std::os::raw::c_int;
use std::ptr;
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

struct ScopeGuard<F: FnMut()> {
    handler: F,
}

impl<F: FnMut()> std::ops::Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        (self.handler)();
    }
}

fn main() {
    unsafe {
        if glfwInit() == 0 {
            panic!("glfwInit");
        }

        let _glfw_guard = ScopeGuard {
            handler: || glfwTerminate(),
        };

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR as _, 3);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR as _, 2);
        glfwWindowHint(GLFW_OPENGL_FORWARD_COMPAT as _, 1);
        glfwWindowHint(GLFW_OPENGL_PROFILE as _, GLFW_OPENGL_CORE_PROFILE as _);
        glfwWindowHint(GLFW_SAMPLES as _, 4);
        glfwWindowHint(GLFW_DOUBLEBUFFER as _, 1);

        let window = glfwCreateWindow(
            1280,
            720,
            b"Chorus Studio\0".as_ptr() as _,
            ptr::null_mut(),
            ptr::null_mut(),
        );

        if window.is_null() {
            println!("glfwCreateWindow");
            glfwTerminate();
            return;
        }

        glfwMakeContextCurrent(window);
        gl::load_with(|s| {
            let cs = std::ffi::CString::new(s).expect("CString::new");
            let ptr = glfwGetProcAddress(cs.as_ptr());
            match ptr {
                Some(p) => p as *const _,
                None => panic!("Failed to load GL func: {}", s),
            }
        });

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
        let mut fonts: [nanovg::Font; Fonts::NumFonts as usize] = std::mem::uninitialized();
        fonts[Fonts::Inter as usize] =
            nanovg::Font::from_file(&nvg, "Inter UI", "assets/Inter-UI-Regular.ttf")
                .expect("Font Inter");
        fonts[Fonts::Vga8 as usize] =
            nanovg::Font::from_file(&nvg, "PxPlus IBM VGA8", "assets/PxPlus_IBM_VGA8.ttf")
                .expect("Font VGA8");
        fonts[Fonts::Moderno as usize] =
            nanovg::Font::from_file(&nvg, "Moderno", "assets/moderno.ttf").expect("Font Moderno");
        {
            let render_ctx = render::RenderContext::new(window, &nvg, fonts);

            while glfwWindowShouldClose(window) == 0 {
                for msg in server_rx.try_iter() {
                    match msg {
                        NetThreadMsg::Connected => {
                            cur_view = Box::new(ui::views::LoginView {
                                username_input: String::new(),
                                password_input: String::new(),
                            });
                        }
                        NetThreadMsg::Response(res) => match res {
                            proto::Response::UserList(users) => {
                                cur_users.replace(users);
                            }
                            proto::Response::LoginOk => {
                                cur_view = Box::new(ui::views::MainView {
                                    user_list: &cur_users,
                                });
                                let _ =
                                    main_tx.send(MainThreadMsg::Command(proto::Command::ListUsers));
                            }
                            proto::Response::LoginInvalid => {}
                        },
                    }
                }

                let (mut w, mut h): (c_int, c_int) = (0, 0);
                glfwGetFramebufferSize(window, &mut w as *mut _, &mut h as *mut _);
                gl::Viewport(0, 0, w, h);
                gl::ClearColor(0.2, 0.4, 0.8, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

                cur_view.present(&render_ctx);
                glfwSwapBuffers(window);
                glfwPollEvents();
            }
        }

        glfwHideWindow(window);

        if let Ok(..) = main_tx.send(MainThreadMsg::Shutdown) {
            let _ = network_thread.join();
        }
    }
}
