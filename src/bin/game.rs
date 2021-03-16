#![allow(clippy::single_match)]
#![feature(or_patterns)]
extern crate erlking;

use erlking::{
    asset::SpriteAsset,
    camera::{ActiveCamera, ParallaxCamera},
    App, Game, KeyboardInput, Position, Rotation, Scale, Sprite,
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
    Idle,
    Walk(Instant),
}

impl PlayerState {
    pub fn animation_state(&self, now: Instant) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Walk(start) => {
                let animation = vec![
                    (7, 0.7),
                    (6, 0.6),
                    (5, 0.5),
                    (4, 0.4),
                    (3, 0.3),
                    (2, 0.2),
                    (1, 0.1),
                    (0, 0.0),
                ];
                let dt = now - *start;
                let dt = dt.as_secs_f32() % 0.8;
                let mut frame = 0;
                for (f, time) in animation {
                    if dt > time {
                        frame = f;
                        break;
                    }
                }
                frame
            }
        }
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let app = futures::executor::block_on(App::new("parallax-demo", &event_loop));
    let mut parallax_demo = Game::new();

    let sprite_assets = vec![
        SpriteAsset::new("player", vec![
            "assets/run0.png",
            "assets/run1.png",
            "assets/run2.png",
            "assets/run3.png",
            "assets/run4.png",
            "assets/run5.png",
            "assets/run6.png",
            "assets/run7.png",
        ]),
        SpriteAsset::new("apple", vec!["assets/apple.png"]),
        SpriteAsset::new("ashberry", vec!["assets/ashberry.png"]),
        SpriteAsset::new("baobab", vec!["assets/baobab.png"]),
        SpriteAsset::new("beech", vec!["assets/beech.png"]),
    ];

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
        PlayerState::Idle,
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
                        PlayerState::Idle => *state = PlayerState::Walk(instant),
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
                        PlayerState::Idle => *state = PlayerState::Walk(instant),
                        _ => (),
                    }
                    pos.0 += dx;
                }
                _ => *state = PlayerState::Idle,
            }
        } else {
            *state = PlayerState::Idle;
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
    let mut q = world.query::<(&PlayerState, &mut Sprite)>();

    for (_, (state, sprite)) in q.iter() {
        sprite.frame_id = state.animation_state(instant);
    }
}
