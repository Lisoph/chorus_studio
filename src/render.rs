use glfw_ffi::*;
use nanovg;

use std::os::raw::c_int;
use std::ptr;

#[repr(usize)]
#[derive(PartialEq, Eq)]
pub enum Fonts {
    Inter = 0,
    Vga8,
    Moderno,
    NumFonts,
}

pub struct RenderContext<'a> {
    window: *mut GLFWwindow,
    nvg: &'a nanovg::Context,
    fonts: [nanovg::Font<'a>; Fonts::NumFonts as usize],
}

impl<'a> RenderContext<'a> {
    pub fn new(
        window: *mut GLFWwindow,
        nvg: &'a nanovg::Context,
        fonts: [nanovg::Font<'a>; Fonts::NumFonts as usize],
    ) -> Self {
        Self { window, nvg, fonts }
    }

    pub fn size(&self) -> (f32, f32) {
        let (mut w, mut h) = (0i32, 0i32);
        unsafe {
            glfwGetWindowSize(self.window, &mut w as *mut _, &mut h as *mut _);
        }
        (w as f32, h as f32)
    }

    pub fn pixel_ratio(&self) -> f32 {
        unsafe {
            let mut fb_width: c_int = 0;
            let mut win_width: c_int = 0;
            glfwGetFramebufferSize(self.window, &mut fb_width as *mut _, ptr::null_mut());
            glfwGetWindowSize(self.window, &mut win_width as *mut _, ptr::null_mut());
            fb_width as f32 / win_width as f32
        }
    }

    pub fn frame<F: FnOnce(nanovg::Frame)>(&self, f: F) {
        self.nvg.frame(self.size(), self.pixel_ratio(), f);
    }

    pub fn font(&self, id: Fonts) -> nanovg::Font<'a> {
        if id == Fonts::NumFonts {
            panic!("Tried to access font `Fonts::NumFonts`");
        }

        self.fonts[id as usize]
    }
}
