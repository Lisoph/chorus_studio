use nanovg::{Color, TextOptions};
use proto;

use render::{Fonts, RenderContext};

use std::cell::RefCell;

pub struct MainLoadingView<'a> {
    pub cur_load_task: &'a RefCell<String>,
}

impl<'a> super::View for MainLoadingView<'a> {
    fn present(&mut self, ctx: &RenderContext) {
        let (w, h) = ctx.size();
        ctx.frame(|f| {
            f.path(
                |p| {
                    let radius = 30.0;
                    p.circle((w / 2.0, h / 2.0), radius);
                    p.fill(Color::from_rgb(200, 100, 0), Default::default());
                },
                Default::default(),
            );
            // Chorus Studio
            {
                let size = 60.0;
                let text = "Chorus Studio";
                let font = ctx.font(Fonts::Moderno);
                let (_, bounds) = f.text_bounds(
                    font,
                    (0.0, 0.0),
                    text,
                    TextOptions {
                        size: size,
                        ..Default::default()
                    },
                );
                let (width, height) = (bounds.max_x, bounds.max_y);
                f.text(
                    font,
                    ((w - width) / 2.0, (h - height) / 8.0),
                    text,
                    TextOptions {
                        size: size,
                        color: Color::from_rgb(255, 255, 255),
                        ..Default::default()
                    },
                );
            }
            // loading
            {
                let size = 28.0;
                let font = ctx.font(Fonts::Vga8);
                let (_, bounds) = f.text_bounds(
                    font,
                    (0.0, 0.0),
                    &*self.cur_load_task.borrow(),
                    TextOptions {
                        size: size,
                        ..Default::default()
                    },
                );
                let (width, height) = (bounds.max_x, bounds.max_y);
                f.text(
                    font,
                    ((w - width) / 2.0, (h - height) / 4.0),
                    &*self.cur_load_task.borrow(),
                    TextOptions {
                        size: size,
                        color: Color::from_rgb(200, 200, 200),
                        ..Default::default()
                    },
                );
            }
        });
    }
}

pub struct MainView<'a> {
    pub user_list: &'a RefCell<Vec<proto::User>>,
}

impl<'a> super::View for MainView<'a> {
    fn present(&mut self, ctx: &RenderContext) {
        ctx.frame(|f| {
            let mut cur_y = 50.0;
            let size = 24.0;

            f.text(
                ctx.font(Fonts::Moderno),
                (5.0, 5.0),
                "Chorus Studio",
                TextOptions {
                    size: 40.0,
                    color: Color::from_rgb(255, 255, 255),
                    ..Default::default()
                },
            );

            for user in self.user_list.borrow().iter() {
                f.text(
                    ctx.font(Fonts::Vga8),
                    (10.0, cur_y),
                    &user.name,
                    TextOptions {
                        size: size,
                        color: Color::from_rgb(255, 255, 255),
                        ..Default::default()
                    },
                );
                f.text(
                    ctx.font(Fonts::Inter),
                    (200.0, cur_y + 7.0),
                    match user.status {
                        proto::UserStatus::Avail => "online",
                        proto::UserStatus::Away => "away",
                        proto::UserStatus::Offline => "offline",
                    },
                    TextOptions {
                        size: 14.0,
                        color: Color::from_rgb(155, 155, 155),
                        ..Default::default()
                    },
                );
                f.text(
                    ctx.font(Fonts::Inter),
                    (250.0, cur_y + 7.0),
                    &if let Some(ref project) = user.in_project {
                        format!("In project '{}'", project)
                    } else {
                        "Chatting".to_owned()
                    },
                    TextOptions {
                        size: 14.0,
                        color: Color::from_rgb(155, 155, 155),
                        ..Default::default()
                    },
                );
                cur_y += size;
            }
        });
    }
}
