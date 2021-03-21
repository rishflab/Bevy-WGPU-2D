#![allow(clippy::single_match)]
#![feature(or_patterns)]
extern crate erlking;

use erlking::sprite::{AnimTimeline, KeyFrame, Sprite};
use erlking::{
    asset::SpriteData,
    camera::{ActiveCamera, ParallaxCamera},
    App, Game, KeyboardInput, Position, Rotation, Scale,
};
use glam::{Quat, Vec3};
use hecs::World;
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, VirtualKeyCode},
    event_loop::EventLoop,
};

#[derive(Clone, Copy)]
struct MoveSpeed(f32);

enum PlayerState {
    Idle(Instant),
    Run(Instant),
}

impl PlayerState {
    pub fn animation_state(&self, now: Instant, timeline: &AnimTimeline) -> u8 {
        match self {
            Self::Idle(start) => {
                let dt = now - *start;
                timeline.current_frame(0..8, 0.8, dt.as_secs_f32())
            }
            Self::Run(start) => {
                let dt = now - *start;
                timeline.current_frame(8..16, 0.8, dt.as_secs_f32())
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let app = futures::executor::block_on(App::new("parallax-demo", &event_loop));
    let mut parallax_demo = Game::new();

    let sprite_assets = vec![
        SpriteData::load_from_anim_strips("player", vec![
            "assets/huntress/idle.png",
            "assets/huntress/run.png",
        ]),
        SpriteData::load("apple", vec!["assets/apple.png"]),
        SpriteData::load("ashberry", vec!["assets/ashberry.png"]),
        SpriteData::load("baobab", vec!["assets/baobab.png"]),
        SpriteData::load("beech", vec!["assets/beech.png"]),
    ];

    let anim_timeline = AnimTimeline::new(
        vec![
            KeyFrame {
                index: 0,
                time: 0.0,
            },
            KeyFrame {
                index: 1,
                time: 0.1,
            },
            KeyFrame {
                index: 2,
                time: 0.2,
            },
            KeyFrame {
                index: 3,
                time: 0.3,
            },
            KeyFrame {
                index: 4,
                time: 0.4,
            },
            KeyFrame {
                index: 5,
                time: 0.5,
            },
            KeyFrame {
                index: 6,
                time: 0.6,
            },
            KeyFrame {
                index: 7,
                time: 0.7,
            },
            KeyFrame {
                index: 8,
                time: 0.0,
            },
            KeyFrame {
                index: 9,
                time: 0.1,
            },
            KeyFrame {
                index: 10,
                time: 0.2,
            },
            KeyFrame {
                index: 11,
                time: 0.3,
            },
            KeyFrame {
                index: 12,
                time: 0.4,
            },
            KeyFrame {
                index: 13,
                time: 0.5,
            },
            KeyFrame {
                index: 14,
                time: 0.6,
            },
            KeyFrame {
                index: 15,
                time: 0.7,
            },
        ]
        .into_iter(),
    );

    let movespeed = MoveSpeed(10.0);

    let camera = (
        ParallaxCamera::new(
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
            0.1,
            500.0,
        ),
        KeyboardInput(None),
        ActiveCamera,
        movespeed,
    );

    let player = (
        Position(Vec3::new(0.0, 0.0, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        KeyboardInput(None),
        Sprite::new("player"),
        anim_timeline,
        PlayerState::Idle(Instant::now()),
        movespeed,
    );

    let apple = (
        Position(Vec3::new(-2.0, 0.0, 30.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new("apple"),
    );

    let ashberry = (
        Position(Vec3::new(2.0, 0.0, 30.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new("ashberry"),
    );

    let baobab = (
        Position(Vec3::new(3.0, 0.0, 55.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new("baobab"),
    );

    let beech = (
        Position(Vec3::new(-3.5, 0.0, 95.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new("beech"),
    );

    parallax_demo.spawn_entity(player);
    parallax_demo.spawn_entity(apple);
    parallax_demo.spawn_entity(ashberry);
    parallax_demo.spawn_entity(baobab);
    parallax_demo.spawn_entity(beech);
    parallax_demo.spawn_entity(camera);

    parallax_demo.add_system(&move_player);
    parallax_demo.add_system(&move_camera);
    parallax_demo.add_system(&update_animation_state);

    app.run(event_loop, parallax_demo, sprite_assets);
}

fn move_player(world: &World, dt: Duration, instant: Instant) {
    let mut q = world.query::<(&KeyboardInput, &mut Position, &MoveSpeed, &mut PlayerState)>();

    for (_, (key, pos, speed, state)) in q.iter() {
        if let Some(input) = key.0 {
            let dx = Vec3::new(speed.0 * dt.as_secs_f32(), 0.0, 0.0);
            match input {
                winit::event::KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                } => {
                    match state {
                        PlayerState::Idle(..) => *state = PlayerState::Run(instant),
                        _ => (),
                    }
                    pos.0 -= dx;
                }
                winit::event::KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                } => {
                    match state {
                        PlayerState::Idle(..) => *state = PlayerState::Run(instant),
                        _ => (),
                    }
                    pos.0 += dx;
                }
                _ => match state {
                    PlayerState::Run(..) => *state = PlayerState::Idle(instant),
                    _ => (),
                },
            }
        } else {
            match state {
                PlayerState::Run(..) => *state = PlayerState::Idle(instant),
                _ => (),
            }
        }
    }
}

fn move_camera(world: &World, dt: Duration, _instant: Instant) {
    let mut q = world.query::<(
        &ActiveCamera,
        &mut ParallaxCamera,
        &KeyboardInput,
        &MoveSpeed,
    )>();

    let (_, (_, cam, key, speed)) = q.iter().next().expect("active camera is present");
    if let Some(input) = key.0 {
        let dx = Vec3::new(speed.0 * dt.as_secs_f32(), 0.0, 0.0);
        match input {
            winit::event::KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Left),
                ..
            } => {
                cam.eye -= dx;
            }
            winit::event::KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Right),
                ..
            } => {
                cam.eye += dx;
            }
            _ => (),
        }
    }
}

fn update_animation_state(world: &World, _dt: Duration, instant: Instant) {
    let mut q = world.query::<(&PlayerState, &mut Sprite, &AnimTimeline)>();

    for (_, (state, sprite, timeline)) in q.iter() {
        sprite.frame_id = state.animation_state(instant, timeline);
    }
}
