use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

pub enum Command {
    Left,
    Right,
    None,
}

pub type IsDown = bool;

#[derive(Debug, Copy, Clone)]
pub struct KeyState {
    pub left: IsDown,
    pub right: IsDown,
    pub up: IsDown,
    pub down: IsDown,
    pub last_pressed: Option<VirtualKeyCode>,
}

impl KeyState {
    pub fn new() -> Self {
        KeyState {
            left: false,
            right: false,
            up: false,
            down: false,
            last_pressed: None,
        }
    }

    pub fn update(&mut self, input: KeyboardInput) {
        self.last_pressed = input.virtual_keycode;
        match input {
            winit::event::KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Left),
                ..
            } => match state {
                ElementState::Pressed => self.left = true,
                ElementState::Released => self.left = false,
            },
            winit::event::KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Right),
                ..
            } => match state {
                ElementState::Pressed => self.right = true,
                ElementState::Released => self.right = false,
            },
            _ => (),
        }
    }
}

impl Default for KeyState {
    fn default() -> Self {
        Self::new()
    }
}
