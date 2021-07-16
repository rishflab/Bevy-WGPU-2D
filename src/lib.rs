use crate::asset::SpriteId;
use crate::input::KeyState;
use crate::sprite::Sprite;
use crate::{
    camera::{ActiveCamera, Camera, ParallaxCamera},
    time::Timer,
};
pub use app::App;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::prelude::{Schedule, Stage, SystemStage, World};
use bevy_ecs::schedule::SystemDescriptor;
use bevy_ecs::world::SpawnBatchIter;
use glam::{Quat, Vec3};
use renderer::gpu_primitives::{Instance, InstanceRaw};
use renderer::scene::Scene;
pub use renderer::TEXTURE_ARRAY_SIZE;
use winit::event::WindowEvent;

pub mod app;
pub mod asset;
pub mod camera;
pub mod input;
pub mod player;
mod renderer;
pub mod sprite;
mod time;

pub struct Position(pub Vec3);
#[derive(PartialOrd, PartialEq)]
pub struct Velocity(pub Vec3);
pub struct Rotation(pub Quat);
pub struct Scale(pub u8);
pub struct Collider(pub parry2d::shape::Cuboid);
#[derive(Clone, Copy)]
pub struct MoveSpeed(pub f32);
pub struct Terrain;

pub struct Game {
    world: World,
    schedule: Schedule,
}

impl Game {
    pub fn new() -> Game {
        let mut schedule = Schedule::default();
        schedule.add_stage("gameplay", SystemStage::parallel());

        let mut world = World::default();
        world.insert_resource(Timer::new());
        world.insert_resource(KeyState::new());

        Game { world, schedule }
    }

    fn run(&mut self) -> Scene {
        self.world.get_resource_mut::<Timer>().unwrap().tick();
        self.schedule.run(&mut self.world);
        self.build_scene()
    }

    pub fn spawn(&mut self, components: impl Bundle) -> Entity {
        self.world.spawn().insert_bundle(components).id()
    }

    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle,
    {
        self.world.spawn_batch(iter)
    }

    pub fn add_system(&mut self, system: impl Into<SystemDescriptor>) {
        self.schedule.add_system_to_stage("gameplay", system);
    }

    fn capture_input_event(&mut self, event: winit::event::WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            self.world
                .get_resource_mut::<KeyState>()
                .unwrap()
                .update(input)
        }
    }

    fn clear_pressed_with_frame(&mut self) {
        self.world
            .get_resource_mut::<KeyState>()
            .unwrap()
            .pressed_this_frame = None;
    }

    fn build_scene(&mut self) -> Scene {
        let mut sprites: Vec<(SpriteId, InstanceRaw)> = vec![];

        let mut query = self
            .world
            .query::<(&Position, &Rotation, &Scale, &Sprite)>();

        for (pos, rot, scale, sprite) in query.iter_mut(&mut self.world) {
            let instance_raw = InstanceRaw::from(Instance {
                position: pos.0,
                rotation: rot.0,
                scale: Vec3::splat(scale.0 as f32),
                frame_id: sprite.anim_frame_index,
            });
            sprites.push((sprite.id(), instance_raw))
        }

        let mut colliders: Vec<InstanceRaw> = vec![];

        let mut query = self.world.query::<(&Position, &Rotation, &Collider)>();

        for (pos, rot, collider) in query.iter(&self.world) {
            let instance_raw = InstanceRaw::from(Instance {
                position: pos.0,
                rotation: rot.0,
                scale: Vec3::new(
                    2.0 * collider.0.half_extents.x,
                    2.0 * collider.0.half_extents.y,
                    1.0,
                ),
                frame_id: 0,
            });

            colliders.push(instance_raw);
        }

        let mut query = self.world.query::<(&ActiveCamera, &ParallaxCamera)>();

        let (_, cam) = query.iter(&self.world).next().expect("No camera defined");

        Scene {
            sprite_instances: sprites,
            camera_uniform: cam.generate_matrix(),
            hitbox_instances: colliders,
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
