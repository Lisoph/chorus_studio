use glfw_ffi::*;

use std::ops::{Deref, Index, Range};

/// Keyboard key
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct KeyAction {
    pub key: u32,
    pub scancode: u32,
    pub action: u32,
    pub mods: u32,
}

impl KeyAction {
    pub fn was_pressed(&self, keycode: KeyCode) -> bool {
        (self.action == GLFW_PRESS || self.action == GLFW_REPEAT) && self.key == keycode as u32
    }

    pub fn was_pressed_once(&self, keycode: KeyCode) -> bool {
        self.action == GLFW_PRESS && self.key == keycode as u32
    }

    pub fn with_modifier(&self, keymod: KeyMod) -> bool {
        self.mods & keymod as u32 != 0
    }
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Backspace = GLFW_KEY_BACKSPACE,
    Delete = GLFW_KEY_DELETE,
    Return = GLFW_KEY_ENTER,
    Tab = GLFW_KEY_TAB,
    Left = GLFW_KEY_LEFT,
    Right = GLFW_KEY_RIGHT,
    Home = GLFW_KEY_HOME,
    End = GLFW_KEY_END,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeyMod {
    Shift = GLFW_MOD_SHIFT,
}

/// A string type similar to [::std::string::String], but which
/// operates on `char` indices instead of not byte indices.
pub struct InputString {
    chars: Vec<char>,
    string: String,
}

impl InputString {
    pub fn new() -> Self {
        Self {
            chars: Vec::new(),
            string: String::new(),
        }
    }

    pub fn insert(&mut self, idx: usize, ch: char) {
        self.chars.insert(idx, ch);
        let mut buf = [0u8; 4];
        let byte_idx = self.byte_index(idx);
        self.string.insert_str(byte_idx, ch.encode_utf8(&mut buf));
    }

    pub fn remove(&mut self, idx: usize) -> char {
        let ch = self.chars.remove(idx);
        let byte_idx = self.byte_index(idx);
        assert_eq!(self.string.remove(byte_idx), ch);
        ch
    }

    pub fn byte_index(&self, char_index: usize) -> usize {
        self.chars
            .iter()
            .take(char_index)
            .map(|c| c.len_utf8())
            .fold(0, |n, l| n + l)
    }

    pub fn as_str(&self) -> &str {
        &self.string
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }
}

impl Index<Range<usize>> for InputString {
    type Output = str;
    fn index(&self, index: Range<usize>) -> &Self::Output {
        let (start, end) = (self.byte_index(index.start), self.byte_index(index.end));
        &self.as_str()[start..end]
    }
}

impl Deref for InputString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
