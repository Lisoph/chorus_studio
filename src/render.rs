use nanovg;
use sdl2;

#[repr(usize)]
#[derive(PartialEq, Eq)]
pub enum Fonts {
    Inter = 0,
    Vga8,
    Moderno,
    NumFonts,
}

pub struct RenderContext<'a> {
    window: &'a sdl2::video::Window,
    nvg: &'a nanovg::Context,
    fonts: [nanovg::Font<'a>; Fonts::NumFonts as usize],
}

impl<'a> RenderContext<'a> {
    pub fn new(
        window: &'a sdl2::video::Window,
        nvg: &'a nanovg::Context,
        fonts: [nanovg::Font<'a>; Fonts::NumFonts as usize],
    ) -> Self {
        Self { window, nvg, fonts }
    }

    pub fn size(&self) -> (f32, f32) {
        let (w, h) = self.window.size();
        (w as f32, h as f32)
    }

    pub fn dpi(&self) -> f32 {
        self.window
            .display_index()
            .and_then(|i| self.window.subsystem().display_dpi(i))
            .map(|v| v.0 / 96.0)
            .unwrap_or(1.0)
    }

    pub fn frame<F: FnOnce(nanovg::Frame)>(&self, f: F) {
        self.nvg.frame(self.size(), self.dpi(), f);
    }

    pub fn font(&self, id: Fonts) -> nanovg::Font<'a> {
        if id == Fonts::NumFonts {
            panic!("Tried to access font Fonts::NumFonts");
        }

        self.fonts[id as usize]
    }
}
