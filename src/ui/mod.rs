pub mod views;

use render::RenderContext;

pub trait View {
    fn present(&mut self, ctx: &RenderContext);
}
