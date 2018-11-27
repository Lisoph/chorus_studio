use nanovg::{Alignment, Color, TextOptions};
use proto;
use sha3::{Digest, Sha3_256};

use input::{KeyAction, KeyCode, KeyMod};
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

pub struct LoginView<'a> {
    username_input: String,
    password_input: String,
    username_cursor: usize,
    password_cursor: usize,
    active_input: LoginViewActiveInput,
    on_submit: Box<FnMut(&str, &[u8]) + 'a>,
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum LoginViewActiveInput {
    Username,
    Password,
}

impl<'a> LoginView<'a> {
    pub fn new(on_submit: Box<FnMut(&str, &[u8]) + 'a>) -> Self {
        LoginView {
            username_input: String::new(),
            password_input: String::new(),
            username_cursor: 0,
            password_cursor: 0,
            active_input: LoginViewActiveInput::Username,
            on_submit,
        }
    }

    fn active_input_data(&mut self) -> (&mut String, &mut usize) {
        match self.active_input {
            LoginViewActiveInput::Username => (&mut self.username_input, &mut self.username_cursor),
            LoginViewActiveInput::Password => (&mut self.password_input, &mut self.password_cursor),
        }
    }
}

impl<'a> super::View for LoginView<'a> {
    fn present(&mut self, ctx: &RenderContext) {
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

            // Button contents

            let placeholder_color = Color::from_rgb(128, 128, 128);
            let content_color = Color::from_rgb(255, 255, 255);

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
                let end_opt = str_char_at(&string, *cursor);
                let end = end_opt.unwrap_or(string.len());
                let (adv, _bounds) = f.text_bounds(
                    ctx.font(Fonts::Inter),
                    origin,
                    &string[0..end],
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

                // Debug string
                f.text(
                    ctx.font(Fonts::Vga8),
                    (10.0, 10.0),
                    format!("Char: {}, byte: {:?}", *cursor, end_opt),
                    TextOptions {
                        color: Color::from_rgb(0, 255, 0),
                        size: 12.0,
                        ..Default::default()
                    },
                );
            }
        });
    }

    fn on_char_input(&mut self, c: char) {
        if !c.is_control() {
            let (string, cursor) = self.active_input_data();
            let idx = str_char_after(&string, *cursor);
            string.insert(idx, c);
            *cursor += 1;
        }
    }

    fn on_key_input(&mut self, key: KeyAction) {
        if key.was_pressed(KeyCode::Backspace) {
            let (string, cursor) = self.active_input_data();
            if *cursor > 0 {
                if let Some(idx) = str_char_at(&string, *cursor - 1) {
                    string.remove(idx);
                    *cursor -= 1;
                }
            }
        } else if key.was_pressed(KeyCode::Delete) {
            let (string, cursor) = self.active_input_data();
            if let Some(idx) = str_char_at(&string, *cursor) {
                string.remove(idx);
            }
        } else if key.was_pressed_once(KeyCode::Return) {
            if self.active_input == LoginViewActiveInput::Username {
                self.active_input = LoginViewActiveInput::Password;
            } else if self.active_input == LoginViewActiveInput::Password {
                (self.on_submit)(
                    &self.username_input,
                    Sha3_256::digest(self.password_input.as_bytes()).as_slice(),
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
            let cursor = self.active_input_data().1;
            if *cursor > 0 {
                *cursor -= 1;
            }
        } else if key.was_pressed(KeyCode::Right) {
            let (string, cursor) = self.active_input_data();
            if *cursor < str_num_chars(string) {
                *cursor += 1;
            }
        } else if key.was_pressed(KeyCode::Home) {
            let (_, cursor) = self.active_input_data();
            *cursor = 0;
        } else if key.was_pressed(KeyCode::End) {
            let (string, cursor) = self.active_input_data();
            *cursor = str_num_chars(&string);
        }
    }
}

/// Compute byte (index, length) for the `char_index` of string `s`.
fn str_char_index<S: AsRef<str>>(s: S, char_index: usize) -> (usize, usize) {
    let s = s.as_ref();
    let (mut i, _) = match s.char_indices().nth(char_index) {
        Some(ci) => ci,
        None => return (s.len(), 0),
    };
    let start = i;
    i += 1;
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    (start, i - start)
}

fn str_char_after<S: AsRef<str>>(s: S, char_index: usize) -> usize {
    let (i, _) = str_char_index(s, char_index);
    i
}

fn str_char_at<S: AsRef<str>>(s: S, char_index: usize) -> Option<usize> {
    let s = s.as_ref();
    let (i, _) = str_char_index(s, char_index);
    if i < s.len() {
        Some(i)
    } else {
        None
    }
}

fn str_num_chars<S: AsRef<str>>(s: S) -> usize {
    let s = s.as_ref();
    s.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_char_index() {
        assert_eq!(str_char_index("ABCD", 0), (0, 1));
        assert_eq!(str_char_index("ABCD", 1), (1, 1));
        assert_eq!(str_char_index("ABCD", 3), (3, 1));
        assert_eq!(str_char_index("ABCD", 4), (4, 0));
        assert_eq!(str_char_index("AÖBC", 0), (0, 1));
        assert_eq!(str_char_index("AÖBC", 1), (1, 2));
        assert_eq!(str_char_index("AÖBC", 2), (3, 1));
        assert_eq!(str_char_index("AÖBC", 4), (5, 0));
    }
}
