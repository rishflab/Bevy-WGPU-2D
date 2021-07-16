#![allow(clippy::single_match)]
extern crate erlking;

use bevy_ecs::prelude::IntoSystem;
use erlking::asset::{SpriteId, SpriteRegistry};
use erlking::camera::update_camera_position;
use erlking::player::{
    flip_sprite, get_input_from_keystate, move_players, update_animation_state,
    update_player_state_machine, PlayerInput, PlayerState,
};
use erlking::sprite::Sprite;
use erlking::{
    asset::SpriteData,
    camera::{ActiveCamera, ParallaxCamera},
    App, Collider, Game, MoveSpeed, Position, Rotation, Scale, Terrain, Velocity,
};
use glam::{Quat, Vec3};
use parry2d::na::Vector2;
use parry2d::shape::Cuboid;
use std::time::Instant;
use winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::new();
    let app = futures::executor::block_on(App::new("erlking", &event_loop));

    let mut sprite_registry = SpriteRegistry::new();

    let apple_sprite = sprite_registry.insert(SpriteData::load("apple", vec!["assets/apple.png"]));
    let ashberry_sprite =
        sprite_registry.insert(SpriteData::load("ashberry", vec!["assets/ashberry.png"]));
    let baobab_sprite =
        sprite_registry.insert(SpriteData::load("baobab", vec!["assets/baobab.png"]));
    let beech_sprite = sprite_registry.insert(SpriteData::load("beech", vec!["assets/beech.png"]));
    let dark_block_sprite = sprite_registry.insert(SpriteData::load(
        "dark_block",
        vec!["assets/dark_block.png"],
    ));

    let (anim_timeline, player_sprite_data) =
        SpriteData::load_from_json("player", "assets/huntress/animated_sprite.json");

    let player_sprite = sprite_registry.insert(player_sprite_data);

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

    let mut game = Game::new();

    let player = (
        Position(Vec3::new(0.0, 0.2, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Velocity(Vec3::new(0.0, 0.0, 0.0)),
        Scale(1),
        Sprite::new(player_sprite),
        anim_timeline,
        PlayerInput::None,
        PlayerState::Standing(Instant::now()),
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

    game.spawn(player);
    game.spawn(apple);
    game.spawn(ashberry);
    game.spawn(baobab);
    game.spawn(beech);
    game.spawn(camera);

    game.spawn_batch(floor(dark_block_sprite));

    game.add_system(get_input_from_keystate.system());
    game.add_system(update_player_state_machine.system());
    game.add_system(update_camera_position.system());
    game.add_system(update_animation_state.system());
    game.add_system(flip_sprite.system());
    game.add_system(move_players.system());

    app.run(event_loop, game, sprite_registry);
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
