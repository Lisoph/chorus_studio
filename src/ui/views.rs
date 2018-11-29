use nanovg::{Alignment, Color, TextOptions};
use proto;
use sha3::{Digest, Sha3_256};

use input::{InputString, KeyAction, KeyCode, KeyMod};
use render::{Fonts, RenderContext};

use std::cell::RefCell;
use std::time::{Duration, Instant};

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

pub struct LoginView<'a> {
    username_input: InputString,
    password_input: InputString,
    username_cursor: usize,
    password_cursor: usize,
    active_input: LoginViewActiveInput,
    on_submit: Box<FnMut(&str, &[u8]) + 'a>,
    invalid_timer_start: Option<Instant>,
    invalid_attempts: usize,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum LoginViewActiveInput {
    Username,
    Password,
}

impl<'a> LoginView<'a> {
    pub fn new(on_submit: Box<FnMut(&str, &[u8]) + 'a>) -> Self {
        LoginView {
            username_input: InputString::new(),
            password_input: InputString::new(),
            username_cursor: 0,
            password_cursor: 0,
            active_input: LoginViewActiveInput::Username,
            on_submit,
            invalid_timer_start: None,
            invalid_attempts: 0,
        }
    }

    pub fn invalid_login(&mut self) {
        self.invalid_timer_start = Some(Instant::now());
        self.invalid_attempts += 1;
    }

    fn active_input_data(&mut self) -> (&mut InputString, &mut usize) {
        match self.active_input {
            LoginViewActiveInput::Username => (&mut self.username_input, &mut self.username_cursor),
            LoginViewActiveInput::Password => (&mut self.password_input, &mut self.password_cursor),
        }
    }
}

impl<'a> super::View for LoginView<'a> {
    fn present(&mut self, ctx: &RenderContext) {
        if let Some(inst) = self.invalid_timer_start.clone() {
            if inst.elapsed() > Duration::from_secs(3) {
                self.invalid_timer_start = None;
            }
        }

        let (w, h) = ctx.size();
        ctx.frame(|f| {
            // "Login"
            f.text(
                ctx.font(Fonts::Moderno),
                (w / 2.0, h / 4.0),
                "Login",
                TextOptions {
                    align: Alignment::new().center().middle(),
                    size: 60.0,
                    color: Color::from_rgb(255, 255, 255),
                    ..Default::default()
                },
            );

            // Button shapes
            let input_width = w / 3.0;
            let input_height = 40f32;
            let input_vert_dist = 20f32;
            f.path(
                |p| {
                    p.rounded_rect(
                        (w / 3.0, h / 2.0 - input_height - input_vert_dist / 2.0),
                        (input_width, input_height),
                        5.0,
                    );
                    p.rounded_rect(
                        (w / 3.0, h / 2.0 + input_vert_dist / 2.0),
                        (input_width, input_height),
                        5.0,
                    );
                    p.fill(Color::from_rgb(128, 30, 80), Default::default());
                },
                Default::default(),
            );

            // Invalid credentials text

            if self.invalid_attempts > 0 {
                f.text(
                    ctx.font(Fonts::Inter),
                    (w / 2.0, h / 2.0 + 100.0),
                    "Your username and / or password is wrong. Please double check.",
                    TextOptions {
                        align: Alignment::new().center().middle(),
                        size: 14.0,
                        color: Color::from_rgb(255, 150, 150),
                        ..Default::default()
                    },
                )
            }

            // Button contents

            let (placeholder_color, content_color) = {
                if self.invalid_timer_start.is_some() {
                    (
                        Color::from_rgb(255, 150, 150),
                        Color::from_rgb(255, 150, 150),
                    )
                } else {
                    (
                        Color::from_rgb(128, 128, 128),
                        Color::from_rgb(255, 255, 255),
                    )
                }
            };

            // Username content
            f.text(
                ctx.font(Fonts::Inter),
                (
                    w / 3.0 + 4.0,
                    h / 2.0 - input_height / 2.0 - input_vert_dist / 2.0,
                ),
                if self.username_input.is_empty() {
                    "Username"
                } else {
                    &self.username_input
                },
                TextOptions {
                    align: Alignment::new().left().middle(),
                    size: input_height - 4.0,
                    color: if self.username_input.is_empty() {
                        placeholder_color
                    } else {
                        content_color
                    },
                    ..Default::default()
                },
            );

            // Password content
            f.text(
                ctx.font(Fonts::Inter),
                (
                    w / 3.0 + 4.0,
                    h / 2.0 + input_height / 2.0 + input_vert_dist / 2.0,
                ),
                if self.password_input.is_empty() {
                    "Password"
                } else {
                    &self.password_input
                },
                TextOptions {
                    align: Alignment::new().left().middle(),
                    size: input_height - 4.0,
                    color: if self.password_input.is_empty() {
                        placeholder_color
                    } else {
                        content_color
                    },
                    ..Default::default()
                },
            );

            // Input cursor
            {
                let origin = match self.active_input {
                    LoginViewActiveInput::Username => {
                        (w / 3.0, h / 2.0 - input_height - input_vert_dist / 2.0)
                    }
                    LoginViewActiveInput::Password => (w / 3.0, h / 2.0 + input_vert_dist / 2.0),
                };

                let (string, cursor) = self.active_input_data();
                let (adv, _bounds) = f.text_bounds(
                    ctx.font(Fonts::Inter),
                    origin,
                    &string[0..*cursor],
                    TextOptions {
                        align: Alignment::new().left().middle(),
                        size: input_height - 4.0,
                        ..Default::default()
                    },
                );
                f.path(
                    |p| {
                        p.rect((origin.0 + adv, origin.1 + 2.0), (2.0, input_height - 4.0));
                        p.fill(Color::from_rgb(255, 0, 0), Default::default());
                    },
                    Default::default(),
                );
            }
        });
    }

    fn on_char_input(&mut self, c: char) {
        if !c.is_control() {
            let (string, cursor) = self.active_input_data();
            string.insert(*cursor, c);
            *cursor += 1;
        }
    }

    fn on_key_input(&mut self, key: KeyAction) {
        if key.was_pressed(KeyCode::Backspace) {
            let (string, cursor) = self.active_input_data();
            if *cursor > 0 {
                string.remove(*cursor - 1);
                *cursor -= 1;
            }
        } else if key.was_pressed(KeyCode::Delete) {
            let (string, cursor) = self.active_input_data();
            if *cursor < string.len() {
                string.remove(*cursor);
            }
        } else if key.was_pressed_once(KeyCode::Return) {
            if self.active_input == LoginViewActiveInput::Username {
                self.active_input = LoginViewActiveInput::Password;
            } else if self.active_input == LoginViewActiveInput::Password {
                (self.on_submit)(
                    &self.username_input.as_str(),
                    Sha3_256::digest(self.password_input.as_str().as_bytes()).as_slice(),
                );
            }
        } else if key.was_pressed_once(KeyCode::Tab) && key.with_modifier(KeyMod::Shift) {
            if self.active_input == LoginViewActiveInput::Password {
                self.active_input = LoginViewActiveInput::Username;
            }
        } else if key.was_pressed_once(KeyCode::Tab) {
            if self.active_input == LoginViewActiveInput::Username {
                self.active_input = LoginViewActiveInput::Password;
            }
        } else if key.was_pressed(KeyCode::Left) {
            let (_string, cursor) = self.active_input_data();
            if *cursor > 0 {
                *cursor -= 1;
            }
        } else if key.was_pressed(KeyCode::Right) {
            let (string, cursor) = self.active_input_data();
            if *cursor < string.len() {
                *cursor += 1;
            }
        } else if key.was_pressed(KeyCode::Home) {
            let (_, cursor) = self.active_input_data();
            *cursor = 0;
        } else if key.was_pressed(KeyCode::End) {
            let (string, cursor) = self.active_input_data();
            *cursor = string.len();
        }
    }
}
