#![allow(clippy::single_match)]
extern crate erlking;

use erlking::asset::{SpriteId, SpriteRegistry};
use erlking::camera::update_camera_position;

use erlking::player::{get_input_from_keystate, update_animation_state, PlayerInput, PlayerState};
use erlking::sprite::Sprite;
use erlking::{
    asset::SpriteData,
    camera::{ActiveCamera, ParallaxCamera},
    App, Collider, Game, Position, Resources, Rotation, Scale,
};
use glam::{Quat, Vec3};
use hecs::World;
use parry2d::math::Isometry;
use parry2d::na::Vector2;
use parry2d::query::TOIStatus;
use parry2d::shape::Cuboid;
use std::cmp::Ordering;
use winit::event_loop::EventLoop;

#[derive(Clone, Copy)]
struct MoveSpeed(f32);

struct Terrain;

fn main() {
    let event_loop = EventLoop::new();
    let app = futures::executor::block_on(App::new("parallax-demo", &event_loop));
    let mut parallax_demo = Game::new();

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

    let player = (
        Position(Vec3::new(0.0, 0.2, 20.0)),
        Rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0)),
        Scale(1),
        Sprite::new(player_sprite),
        anim_timeline,
        PlayerInput::None,
        PlayerState::Standing(parallax_demo.now()),
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

    parallax_demo.add_system(&get_input_from_keystate);
    parallax_demo.add_system(&update_player_state_machine);
    parallax_demo.add_system(&update_camera_position);
    parallax_demo.add_system(&update_animation_state);

    app.run(event_loop, parallax_demo, sprite_registry);
}

fn update_player_state_machine(world: &World, res: Resources) {
    let mut q = world.query::<(
        &mut PlayerState,
        &mut PlayerInput,
        &mut Position,
        &mut Rotation,
        &MoveSpeed,
        &Collider,
    )>();

    for (_, (state, input, pos, rot, speed, player_collider)) in q.iter() {
        let vel = Vec3::new(speed.0, 0.0, 0.0);
        let (new_state, vel) = state.handle_player_input(vel, &mut rot.0, input, res.now);

        let mut terrain = world.query::<(&Collider, &Position, &Terrain)>();

        let thresh = 0.001;
        let max_toi = res.dt.as_secs_f32();

        let collision = terrain
            .iter()
            .filter_map(|(_, (terrain_collider, terrain_pos, _))| {
                parry2d::query::time_of_impact(
                    &Isometry::translation(terrain_pos.0.x, terrain_pos.0.y),
                    &Vector2::new(0.0, 0.0),
                    &terrain_collider.0,
                    &Isometry::translation(pos.0.x, pos.0.y),
                    &Vector2::new(vel.x, vel.y),
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
                // Eventually only the smalled item remains
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
            match toi.status {
                TOIStatus::Converged => pos.0 += vel * toi.toi,
                _ => (),
            }
        } else {
            pos.0 += vel * max_toi;
        }
        *state = new_state;
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
