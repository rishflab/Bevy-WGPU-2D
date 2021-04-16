use crate::input::KeyState;
use crate::sprite::{AnimTimeline, Sprite};
use crate::Resources;
use glam::{Quat, Vec3};
use hecs::World;
use std::time::Instant;
use winit::event::VirtualKeyCode;

#[derive(Clone, Copy)]
pub enum PlayerState {
    Standing(Instant),
    Running(Instant),
    Attacking(Instant),
}

pub enum PlayerInput {
    Left,
    Right,
    Attack,
    None,
}

impl PlayerState {
    pub fn handle_player_input(
        self,
        vel: Vec3,
        rot: &mut Quat,
        input: &PlayerInput,
        now: Instant,
    ) -> (Self, Vec3) {
        let attack_duration = 0.45;
        match (self, input) {
            (Self::Standing(..), PlayerInput::Left) => {
                *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 180.0_f32.to_radians());
                (PlayerState::Running(now), Vec3::new(-1.0, 0.0, 0.0) * vel)
            }
            (Self::Standing(..), PlayerInput::Right) => {
                *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0);
                (PlayerState::Running(now), vel)
            }
            (Self::Standing(start), PlayerInput::None) => {
                (PlayerState::Standing(start), Vec3::zero())
            }
            (Self::Standing(..), PlayerInput::Attack) => {
                (PlayerState::Attacking(now), Vec3::zero())
            }
            (Self::Running(start), PlayerInput::Left) => {
                *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 180.0_f32.to_radians());
                (PlayerState::Running(start), Vec3::new(-1.0, 0.0, 0.0) * vel)
            }
            (Self::Running(start), PlayerInput::Right) => {
                *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0);
                (PlayerState::Running(start), vel)
            }
            (Self::Running(..), PlayerInput::None) => (PlayerState::Standing(now), Vec3::zero()),
            (Self::Running(..), PlayerInput::Attack) => (PlayerState::Attacking(now), Vec3::zero()),
            (Self::Attacking(start), PlayerInput::Left) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
                    *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 180.0_f32.to_radians());
                    (PlayerState::Running(now), Vec3::new(-1.0, 0.0, 0.0) * vel)
                } else {
                    (PlayerState::Attacking(start), Vec3::zero())
                }
            }
            (Self::Attacking(start), PlayerInput::Right) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
                    *rot = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0);
                    (PlayerState::Running(now), Vec3::new(1.0, 0.0, 0.0) * vel)
                } else {
                    (PlayerState::Attacking(start), Vec3::zero())
                }
            }
            (Self::Attacking(start), PlayerInput::None) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
                    (PlayerState::Standing(now), Vec3::zero())
                } else {
                    (PlayerState::Attacking(start), Vec3::zero())
                }
            }
            (Self::Attacking(start), PlayerInput::Attack) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
                    (PlayerState::Standing(now), Vec3::zero())
                } else {
                    (PlayerState::Attacking(start), Vec3::zero())
                }
            }
        }
    }
}

impl PlayerState {
    pub fn animation_state(&self, now: Instant, timeline: &AnimTimeline) -> u8 {
        match self {
            Self::Standing(start) => {
                let dt = now - *start;
                timeline.current_frame(0, dt.as_secs_f32())
            }
            Self::Running(start) => {
                let dt = now - *start;
                timeline.current_frame(1, dt.as_secs_f32())
            }
            Self::Attacking(start) => {
                let dt = now - *start;
                timeline.current_frame(2, dt.as_secs_f32())
            }
        }
    }
}

pub fn get_input_from_keystate(world: &World, res: Resources) {
    let mut q = world.query::<&mut PlayerInput>();

    for (_, command) in q.iter() {
        let next = match res.key_state {
            KeyState {
                pressed_this_frame: Some(VirtualKeyCode::A),
                ..
            } => PlayerInput::Attack,
            KeyState {
                left: true,
                right: false,
                ..
            } => PlayerInput::Left,
            KeyState {
                left: false,
                right: true,
                ..
            } => PlayerInput::Right,
            KeyState {
                left: true,
                right: true,
                ..
            } => PlayerInput::Right,
            _ => PlayerInput::None,
        };
        *command = next;
    }
}

pub fn update_animation_state(world: &World, res: Resources) {
    let mut q = world.query::<(&PlayerState, &mut Sprite, &AnimTimeline)>();

    for (_, (state, sprite, timeline)) in q.iter() {
        sprite.offset = state.animation_state(res.now, timeline);
    }
}
