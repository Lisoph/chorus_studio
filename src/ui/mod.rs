pub mod views;

use input::KeyAction;
use render::RenderContext;

pub trait View {
    fn present(&mut self, ctx: &RenderContext);
    fn on_char_input(&mut self, _c: char) {}
    fn on_key_input(&mut self, _k: KeyAction) {}
}

pub enum DynamicView<'a> {
    MainLoading(views::MainLoadingView<'a>),
    Main(views::MainView<'a>),
    Login(views::LoginView<'a>),
}

impl<'a> DynamicView<'a> {
    pub fn view(&mut self) -> &mut View {
        match self {
            DynamicView::MainLoading(v) => v,
            DynamicView::Main(v) => v,
            DynamicView::Login(v) => v,
        }
    }
}
