use glfw_ffi::*;

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
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeyMod {
    Shift = GLFW_MOD_SHIFT,
}
