#![allow(clippy::single_match)]
#![feature(or_patterns)]
extern crate erlking;

use erlking::asset::{load_anim_timeline, SpriteId, SpriteRegistry, View};
use erlking::input::{Command, KeyState};
use erlking::sprite::{AnimTimeline, Sprite};
use erlking::{
    asset::SpriteData,
    camera::{ActiveCamera, ParallaxCamera},
    App, Collider, Game, Position, Resources, Rotation, Scale,
};
use glam::{Quat, Vec3};
use hecs::World;
use parry2d::math::Isometry;
use parry2d::na::Vector2;
use parry2d::shape::Cuboid;
use std::time::Instant;
use winit::event_loop::EventLoop;

#[derive(Clone, Copy)]
struct MoveSpeed(f32);

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
                timeline.current_frame(0..8, dt.as_secs_f32())
            }
            Self::Run(start) => {
                let dt = now - *start;
                timeline.current_frame(8..16, dt.as_secs_f32())
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
        Sprite::new(player_sprite),
        anim_timeline,
        Command::None,
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

    parallax_demo.add_system(&get_command_from_keystate);
    parallax_demo.add_system(&apply_command_to_player);
    parallax_demo.add_system(&update_camera_position);
    parallax_demo.add_system(&update_animation_state);

    app.run(event_loop, parallax_demo, sprite_registry);
}

fn apply_command_to_player(world: &World, res: Resources) {
    let mut q = world.query::<(
        &mut PlayerState,
        &mut Command,
        &mut Position,
        &mut Rotation,
        &MoveSpeed,
        &Collider,
    )>();

    for (_, (state, input, pos, rot, speed, player_collider)) in q.iter() {
        let dx = Vec3::new(speed.0 * res.dt.as_secs_f32(), 0.0, 0.0);

        let new = match input {
            Command::Left => {
                match state {
                    PlayerState::Idle(_) => *state = PlayerState::Run(res.now),
                    _ => (),
                }
                Some((
                    pos.0 - dx,
                    Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 180.0_f32.to_radians()),
                ))
            }
            Command::Right => {
                match state {
                    PlayerState::Idle(_) => *state = PlayerState::Run(res.now),
                    _ => (),
                }
                Some((
                    pos.0 + dx,
                    Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0),
                ))
            }
            Command::None => {
                match state {
                    PlayerState::Run(_) => *state = PlayerState::Idle(res.now),
                    _ => (),
                }
                None
            }
        };

        let mut terrain = world.query::<(&Collider, &Position, &Terrain)>();

        if let Some((player_pos, player_rot)) = new {
            let collisions = terrain
                .iter()
                .filter(|(_, (terrain_collider, terrain_pos, _))| {
                    parry2d::query::intersection_test(
                        &Isometry::translation(terrain_pos.0.x, terrain_pos.0.y),
                        &terrain_collider.0,
                        &Isometry::translation(player_pos.x, player_pos.y),
                        &player_collider.0,
                    )
                    .unwrap()
                })
                .count();

            if collisions == 0 {
                pos.0 = player_pos;
                rot.0 = player_rot
            }
        }
    }
}

fn update_camera_position(world: &World, _res: Resources) {
    if let Some((_, (_, pos))) = world.query::<(&PlayerState, &mut Position)>().iter().next() {
        let mut q = world.query::<(&ActiveCamera, &mut ParallaxCamera)>();

        if let Some((_, (_, camera))) = q.iter().next() {
            camera.eye.x = pos.0.x;
        }
    }
}

fn update_animation_state(world: &World, res: Resources) {
    let mut q = world.query::<(&PlayerState, &mut Sprite, &AnimTimeline)>();

    for (_, (state, sprite, timeline)) in q.iter() {
        sprite.frame_id = state.animation_state(res.now, timeline);
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

pub fn get_command_from_keystate(world: &World, res: Resources) {
    let mut q = world.query::<&mut Command>();

    for (_, command) in q.iter() {
        let next = match res.key_state {
            KeyState {
                left: true,
                right: false,
                ..
            } => Command::Left,
            KeyState {
                left: false,
                right: true,
                ..
            } => Command::Right,
            KeyState {
                left: true,
                right: true,
                ..
            } => Command::Right,
            _ => Command::None,
        };
        *command = next;
    }
}
