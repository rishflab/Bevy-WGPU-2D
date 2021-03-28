#![allow(clippy::single_match)]
#![feature(or_patterns)]
extern crate erlking;

use erlking::asset::{load_anim_timeline, SpriteId, SpriteRegistry, View};
use erlking::sprite::{AnimTimeline, Sprite};
use erlking::{
    asset::SpriteData,
    camera::{ActiveCamera, ParallaxCamera},
    App, Collider, Game, KeyboardInput, Position, Rotation, Scale,
};
use glam::{Quat, Vec3};
use hecs::World;
use parry2d::math::Isometry;
use parry2d::na::Vector2;
use parry2d::shape::Cuboid;
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, VirtualKeyCode},
    event_loop::EventLoop,
};

#[derive(Clone, Copy)]
struct MoveSpeed(f32);

#[derive(Debug, Copy, Clone)]
enum Input {
    Left,
    Right,
    None,
}

struct Terrain;

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

    let mut sprite_registry = SpriteRegistry::new();

    let player_sprite = sprite_registry.insert(SpriteData::load_from_anim_strips(
        "player",
        vec!["assets/huntress/idle.png", "assets/huntress/run.png"],
        View {
            x: 55,
            y: 53,
            width: 40,
            height: 50,
        },
    ));
    let apple_sprite = sprite_registry.insert(SpriteData::load("apple", vec!["assets/apple.png"]));
    let ashberry_sprite =
        sprite_registry.insert(SpriteData::load("ashberry", vec!["assets/ashberry.png"]));
    let baobab_sprite =
        sprite_registry.insert(SpriteData::load("baobab", vec!["assets/baobab.png"]));
    let beech_sprite = sprite_registry.insert(SpriteData::load("beech", vec!["assets/beech.png"]));
    let dark_block_sprite = sprite_registry.insert(SpriteData::load("dark_block", vec![
        "assets/dark_block.png",
    ]));

    let anim_timeline = load_anim_timeline("assets/huntress/keyframes.json");

    let movespeed = MoveSpeed(10.0);

    let camera = (
        ParallaxCamera::new(
            Vec3::new(0.0, 3.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            1.0,
            0.1,
            500.0,
        ),
        ActiveCamera,
    );

    let player = (
        Position(Vec3::new(0.0, 0.2, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        KeyboardInput(None),
        Sprite::new(player_sprite),
        anim_timeline,
        Input::None,
        PlayerState::Idle(Instant::now()),
        Collider(Cuboid::new(Vector2::new(0.4, 0.6))),
        movespeed,
    );

    let apple = (
        Position(Vec3::new(-2.0, 0.0, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new(apple_sprite),
        Collider(Cuboid::new(Vector2::new(0.5, 0.5))),
        Terrain,
    );

    let ashberry = (
        Position(Vec3::new(2.0, 0.0, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new(ashberry_sprite),
        Collider(Cuboid::new(Vector2::new(0.5, 0.5))),
        Terrain,
    );

    let baobab = (
        Position(Vec3::new(3.0, 0.0, 55.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new(baobab_sprite),
        Collider(Cuboid::new(Vector2::new(0.5, 0.5))),
        Terrain,
    );

    let beech = (
        Position(Vec3::new(-3.5, 0.0, 95.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new(beech_sprite),
        Collider(Cuboid::new(Vector2::new(0.5, 0.5))),
        Terrain,
    );

    parallax_demo.spawn(player);
    parallax_demo.spawn(apple);
    parallax_demo.spawn(ashberry);
    parallax_demo.spawn(baobab);
    parallax_demo.spawn(beech);
    parallax_demo.spawn(camera);

    parallax_demo.spawn_batch(floor(dark_block_sprite));

    parallax_demo.add_system(&process_keyboard_input);
    parallax_demo.add_system(&apply_input_to_player);
    parallax_demo.add_system(&update_camera_position);
    parallax_demo.add_system(&update_animation_state);

    app.run(event_loop, parallax_demo, sprite_registry);
}

fn process_keyboard_input(world: &World, _dt: Duration, _instant: Instant) {
    let mut q = world.query::<(&KeyboardInput, &mut Input)>();

    for (_, (key, command)) in q.iter() {
        if let Some(input) = key.0 {
            match input {
                winit::event::KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                } => *command = Input::Left,
                winit::event::KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                } => *command = Input::Right,
                _ => *command = Input::None,
            }
        } else {
            *command = Input::None;
        }
    }
}

fn apply_input_to_player(world: &World, dt: Duration, instant: Instant) {
    let mut q = world.query::<(
        &mut PlayerState,
        &mut Input,
        &mut Position,
        &MoveSpeed,
        &Collider,
    )>();

    let mut terrain = world.query::<(&Collider, &Position, &Terrain)>();

    for (_, (state, input, player_pos, speed, collider)) in q.iter() {
        let dx = Vec3::new(speed.0 * dt.as_secs_f32(), 0.0, 0.0);

        let new_pos = match input {
            Input::Left => {
                match state {
                    PlayerState::Idle(_) => *state = PlayerState::Run(instant),
                    _ => (),
                }
                Some(player_pos.0 + dx)
            }
            Input::Right => {
                match state {
                    PlayerState::Idle(_) => *state = PlayerState::Run(instant),
                    _ => (),
                }
                Some(player_pos.0 - dx)
            }
            _ => {
                *state = PlayerState::Idle(instant);
                None
            }
        };

        if let Some(pos) = new_pos {
            let collisions = terrain
                .iter()
                .filter(|(_, (other, other_pos, _))| {
                    parry2d::query::intersection_test(
                        &Isometry::translation(other_pos.0.x, other_pos.0.y),
                        &other.0,
                        &Isometry::translation(pos.x, pos.y),
                        &collider.0,
                    )
                    .unwrap()
                })
                .count();

            if collisions == 0 {
                player_pos.0 = pos;
            }
        }
    }
}

fn update_camera_position(world: &World, _dt: Duration, _instant: Instant) {
    if let Some((_, (_, pos))) = world.query::<(&PlayerState, &mut Position)>().iter().next() {
        let mut q = world.query::<(&ActiveCamera, &mut ParallaxCamera)>();

        if let Some((_, (_, camera))) = q.iter().next() {
            camera.eye.x = pos.0.x;
        }
    }
}

fn update_animation_state(world: &World, _dt: Duration, instant: Instant) {
    let mut q = world.query::<(&PlayerState, &mut Sprite, &AnimTimeline)>();

    for (_, (state, sprite, timeline)) in q.iter() {
        sprite.frame_id = state.animation_state(instant, timeline);
    }
}

fn floor(sprite_id: SpriteId) -> Vec<(Position, Rotation, Scale, Sprite, Collider, Terrain)> {
    (-5..5)
        .map(|i| {
            (
                Position(Vec3::new(1.0 * i as f32, -1.0, 20.0)),
                Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
                Scale(1),
                Sprite::new(sprite_id),
                Collider(Cuboid::new(Vector2::new(0.5, 0.5))),
                Terrain,
            )
        })
        .collect()
}
