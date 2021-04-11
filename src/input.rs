use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Default, Copy, Clone)]
pub struct KeyState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub a: bool,
    pub pressed_this_frame: Option<VirtualKeyCode>,
}

impl KeyState {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn update(&mut self, input: KeyboardInput) {
        self.pressed_this_frame = None;
        match input {
            KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Left),
                ..
            } => match state {
                ElementState::Pressed => {
                    if !self.left {
                        self.pressed_this_frame = input.virtual_keycode;
                    }
                    self.left = true;
                }
                ElementState::Released => self.left = false,
            },
            KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::Right),
                ..
            } => match state {
                ElementState::Pressed => {
                    if !self.right {
                        self.pressed_this_frame = input.virtual_keycode;
                    }
                    self.right = true;
                }
                ElementState::Released => self.right = false,
            },
            KeyboardInput {
                state,
                virtual_keycode: Some(VirtualKeyCode::A),
                ..
            } => match state {
                ElementState::Pressed => {
                    if !self.a {
                        self.pressed_this_frame = input.virtual_keycode;
                    }
                    self.a = true;
                }
                ElementState::Released => self.a = false,
            },
            _ => (),
        }
    }
}
