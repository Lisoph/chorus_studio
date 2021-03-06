extern crate bincode;
extern crate glfw_ffi;
extern crate nanovg;
extern crate proto;
extern crate sha3;

mod gl;
mod input;
mod render;
mod ui;

use glfw_ffi::*;

use std::cell::RefCell;
use std::net;
use std::os::raw::{c_int, c_uint};
use std::ptr;
use std::sync;
use std::thread;
use std::time;
use std::mem::MaybeUninit;

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

struct MainWindowCtx<'a> {
    char_input_handler: Box<dyn Fn(char) + 'a>,
    key_input_handler: Box<dyn Fn(c_int, c_int, c_int, c_int) + 'a>,
}

unsafe extern "C" fn char_callback(window: *mut GLFWwindow, codepoint: c_uint) {
    let ctx = {
        let ptr = glfwGetWindowUserPointer(window) as *mut MainWindowCtx;
        if ptr.is_null() {
            return;
        } else {
            &mut *ptr
        }
    };
    if let Some(c) = std::char::from_u32(codepoint) {
        (ctx.char_input_handler)(c);
    }
}

unsafe extern "C" fn key_callback(
    window: *mut GLFWwindow,
    key: c_int,
    scancode: c_int,
    action: c_int,
    mods: c_int,
) {
    let ctx = {
        let ptr = glfwGetWindowUserPointer(window) as *mut MainWindowCtx;
        if ptr.is_null() {
            return;
        } else {
            &mut *ptr
        }
    };
    (ctx.key_input_handler)(key, scancode, action, mods);
}

fn load_fonts<'a>(nvg: &'a nanovg::Context) -> Result<[nanovg::Font<'a>; render::Fonts::NumFonts as usize], nanovg::CreateFontError> {
    use render::Fonts;
    
    let inter = nanovg::Font::from_file(&nvg, "Inter UI", "assets/Inter-UI-Regular.ttf")?;
    let vga8 = nanovg::Font::from_file(&nvg, "PxPlus IBM VGA8", "assets/PxPlus_IBM_VGA8.ttf")?;
    let moderno = nanovg::Font::from_file(&nvg, "Moderno", "assets/moderno.ttf")?;
    
    let mut fonts = MaybeUninit::uninit();
    let f = fonts.as_mut_ptr() as *mut nanovg::Font;
    let mut fonts_loaded = 0usize;
    
    unsafe {
        let mut assign = |id: Fonts, font: nanovg::Font<'a>| {
            f.add(id as usize).write(font);
            fonts_loaded += 1;
        };
        
        assign(Fonts::Inter, inter);
        assign(Fonts::Vga8, vga8);
        assign(Fonts::Moderno, moderno);
        
        if fonts_loaded == Fonts::NumFonts as usize {
            Ok(fonts.assume_init())
        } else {
            for i in 0..fonts_loaded {
                f.add(i).drop_in_place();
            }
            panic!("Font loading code doesn't load all fonts!")
        }
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
            return;
        }

        // Networking
        let (main_tx, main_rx) = sync::mpsc::channel();
        let (server_tx, server_rx) = sync::mpsc::channel();
        let network_thread = thread::spawn(move || -> Result<(), ()> {
            use std::str::FromStr;
            let server_addr = net::SocketAddr::from_str(SERVER_IP).map_err(|_| ())?;
            
            // Endlessly connect to server:
            let mut stream = loop {
                if let Ok(MainThreadMsg::Shutdown) = main_rx.try_recv() {
                    return Ok(());
                }

                let timeout = time::Duration::from_secs(4);
                if let Ok(s) = net::TcpStream::connect_timeout(&server_addr, timeout) {
                    break s;
                }
            };

            // We've got a connection, notify the main thread.
            server_tx.send(NetThreadMsg::Connected).map_err(|_| ())?;
            glfwPostEmptyEvent(); // Wake up main loop

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
                    glfwPostEmptyEvent(); // Wake up main loop
                }
            }
        });

        // Data the views depend on
        let load_task = RefCell::new("Connecting to server...".to_owned());
        let cur_users = RefCell::new(Vec::new());

        let cur_view: RefCell<ui::DynamicView> =
            RefCell::new(ui::DynamicView::MainLoading(ui::views::MainLoadingView {
                cur_load_task: &load_task,
            }));

        let mut main_window_ctx = MainWindowCtx {
            char_input_handler: Box::new(|c| {
                let mut cur_view = cur_view.borrow_mut();
                cur_view.view().on_char_input(c);
            }),
            key_input_handler: Box::new(|key, scancode, action, mods| {
                let mut cur_view = cur_view.borrow_mut();
                cur_view.view().on_key_input(input::KeyAction {
                    key: key as u32,
                    scancode: scancode as u32,
                    action: action as u32,
                    mods: mods as u32,
                });
            }),
        };

        glfwSetWindowUserPointer(window, &mut main_window_ctx as *mut MainWindowCtx as *mut _);
        glfwSetCharCallback(window, Some(char_callback));
        glfwSetKeyCallback(window, Some(key_callback));

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

        // Fonts
        let fonts = load_fonts(&nvg).expect("Font loading");
        {
            let render_ctx = render::RenderContext::new(window, &nvg, fonts);

            while glfwWindowShouldClose(window) == 0 {
                for msg in server_rx.try_iter() {
                    match msg {
                        NetThreadMsg::Connected => {
                            cur_view.replace(ui::DynamicView::Login(ui::views::LoginView::new(
                                Box::new(|email, password| {
                                    let _ = main_tx.send(MainThreadMsg::Command(
                                        proto::Command::Login {
                                            email: email.to_owned(),
                                            password: password.to_owned(),
                                        },
                                    ));
                                }),
                            )));
                        }
                        NetThreadMsg::Response(res) => match res {
                            proto::Response::UserList(users) => {
                                cur_users.replace(users);
                            }
                            proto::Response::LoginOk => {
                                cur_view.replace(ui::DynamicView::Main(ui::views::MainView {
                                    user_list: &cur_users,
                                }));
                            }
                            proto::Response::LoginInvalid => {
                                let mut cur_view = cur_view.borrow_mut();
                                if let ui::DynamicView::Login(ref mut login) = *cur_view {
                                    login.invalid_login();
                                }
                            }
                        },
                    }
                }

                let (mut w, mut h): (c_int, c_int) = (0, 0);
                glfwGetFramebufferSize(window, &mut w as *mut _, &mut h as *mut _);
                gl::Viewport(0, 0, w, h);
                gl::ClearColor(0.2, 0.4, 0.8, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

                {
                    let mut cur_view = cur_view.borrow_mut();
                    cur_view.view().present(&render_ctx);
                }
                glfwSwapBuffers(window);
                glfwWaitEvents();
            }
        }

        glfwHideWindow(window);

        if let Ok(..) = main_tx.send(MainThreadMsg::Shutdown) {
            let _ = network_thread.join();
        }
    }
}
