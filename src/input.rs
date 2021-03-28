use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Default, Copy, Clone)]
pub struct KeyState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub last_pressed: Option<VirtualKeyCode>,
}

impl KeyState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, input: KeyboardInput) {
        self.last_pressed = input.virtual_keycode;
        match input {
            KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Left),
                ..
            } => match state {
                ElementState::Pressed => self.left = true,
                ElementState::Released => self.left = false,
            },
            KeyboardInput {
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

