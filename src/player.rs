use crate::input::KeyState;
use crate::sprite::{AnimTimeline, Sprite};
use crate::time::Timer;
use crate::{Collider, MoveSpeed, Position, Rotation, Terrain, Velocity};
use bevy_ecs::prelude::{Changed, Query, Res, Without};
use glam::{Quat, Vec3};
use parry2d::math::Isometry;
use parry2d::na::Vector2;
use parry2d::query::TOIStatus;
use std::cmp::Ordering;
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
    pub fn handle_player_input(self, vel: Vec3, input: &PlayerInput, now: Instant) -> (Self, Vec3) {
        let attack_duration = 0.45;
        match (self, input) {
            (Self::Standing(..), PlayerInput::Left) => {
                (PlayerState::Running(now), Vec3::new(-1.0, 0.0, 0.0) * vel)
            }
            (Self::Standing(..), PlayerInput::Right) => (PlayerState::Running(now), vel),
            (Self::Standing(start), PlayerInput::None) => {
                (PlayerState::Standing(start), Vec3::zero())
            }
            (Self::Standing(..), PlayerInput::Attack) => {
                (PlayerState::Attacking(now), Vec3::zero())
            }
            (Self::Running(start), PlayerInput::Left) => {
                (PlayerState::Running(start), Vec3::new(-1.0, 0.0, 0.0) * vel)
            }
            (Self::Running(start), PlayerInput::Right) => (PlayerState::Running(start), vel),
            (Self::Running(..), PlayerInput::None) => (PlayerState::Standing(now), Vec3::zero()),
            (Self::Running(..), PlayerInput::Attack) => (PlayerState::Attacking(now), Vec3::zero()),
            (Self::Attacking(start), PlayerInput::Left) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
                    (PlayerState::Running(now), Vec3::new(-1.0, 0.0, 0.0) * vel)
                } else {
                    (PlayerState::Attacking(start), Vec3::zero())
                }
            }
            (Self::Attacking(start), PlayerInput::Right) => {
                if now.duration_since(start).as_secs_f32() >= attack_duration {
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

pub fn update_player_state_machine(
    mut query: Query<(&mut PlayerState, &PlayerInput, &mut Velocity, &MoveSpeed)>,
    timer: Res<Timer>,
) {
    for (mut state, input, mut vel, speed) in query.iter_mut() {
        let (new_state, new_vel) =
            state.handle_player_input(Vec3::new(speed.0, 0.0, 0.0), &input, timer.now());

        vel.0 = new_vel;
        *state = new_state;
    }
}

pub fn move_players(
    terrain: Query<(&Collider, &Position, &Terrain)>,
    mut players: Query<(&Collider, &mut Position, &Velocity, Without<Terrain>)>,
    timer: Res<Timer>,
) {
    let thresh = 0.001;
    let max_toi = timer.elapsed().as_secs_f32();

    for (player_collider, mut pos, vel, _) in players.iter_mut() {
        let collision = terrain
            .iter()
            .filter_map(|(terrain_collider, terrain_pos, _)| {
                parry2d::query::time_of_impact(
                    &Isometry::translation(terrain_pos.0.x, terrain_pos.0.y),
                    &Vector2::new(0.0, 0.0),
                    &terrain_collider.0,
                    &Isometry::translation(pos.0.x, pos.0.y),
                    &Vector2::new(vel.0.x, vel.0.y),
                    &player_collider.0,
                    max_toi,
                    thresh,
                )
                .unwrap()
            })
            .min_by(|x, y| {
                // min_by() finds the smallest item in an iterator based on a comparison function.
                // We go through the iterator comparing an item with another.
                // If the item is smaller than the one it is being compared to we keep it and discard the larger item.
                // Eventually only the smallest item remains
                // Below we are comparing the toi, the time-of-impact of the collision.
                // We want to find the collision that happened first ie. had the smallest toi.
                if x.toi > y.toi {
                    Ordering::Less
                } else if x.toi < y.toi {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            });

        if let Some(toi) = collision {
            // TOIStatus has a TOIStatus::Penetrating variant.
            // We do not want to move the player if the player collider is already penetrating terrain.
            if let TOIStatus::Converged = toi.status {
                pos.0 += vel.0 * toi.toi;
            }
        } else {
            pos.0 += vel.0 * max_toi;
        }
    }
}

pub fn get_input_from_keystate(mut query: Query<&mut PlayerInput>, key_state: Res<KeyState>) {
    for mut command in query.iter_mut() {
        let next = match *key_state {
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

pub fn update_animation_state(
    mut query: Query<(&PlayerState, &mut Sprite, &AnimTimeline)>,
    timer: Res<Timer>,
) {
    for (state, mut sprite, timeline) in query.iter_mut() {
        sprite.offset = state.animation_state(timer.now(), timeline);
    }
}

pub fn flip_sprite(mut query: Query<(Changed<Velocity>, &Velocity, &mut Rotation, &Sprite)>) {
    for (vel_changed, vel, mut rot, _) in query.iter_mut() {
        if vel_changed {
            if vel.0.x > 0.0 {
                rot.0 = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0);
            } else if vel.0.x < 0.0 {
                rot.0 = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 180.0_f32.to_radians());
            }
        }
    }
}
