pub mod views;

use input::KeyAction;
use render::RenderContext;

pub trait View {
    fn present(&mut self, ctx: &RenderContext);
    fn on_char_input(&mut self, _c: char) {}
    fn on_key_input(&mut self, _k: KeyAction) {}
}
