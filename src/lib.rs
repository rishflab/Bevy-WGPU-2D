#![feature(in_band_lifetimes)]

use crate::asset::SpriteId;
use crate::sprite::Sprite;
use crate::{
    camera::{ActiveCamera, Camera, ParallaxCamera},
    time::Timer,
};
pub use app::App;
use glam::{Quat, Vec3};
use hecs::{Bundle, DynamicBundle, Entity, SpawnBatchIter, World};
use renderer::gpu_primitives::{Instance, InstanceRaw};
use renderer::scene::Scene;
pub use renderer::TEXTURE_ARRAY_SIZE;
use std::time::{Duration, Instant};
use winit::event::WindowEvent;

pub mod app;
pub mod asset;
pub mod camera;
mod renderer;
pub mod sprite;
mod time;

pub struct Position(pub Vec3);
pub struct Rotation(pub Quat);
pub struct Scale(pub u8);
pub struct KeyboardInput(pub Option<winit::event::KeyboardInput>);
pub struct Collider(pub parry2d::shape::Cuboid);

pub struct Game<'a> {
    world: World,
    timer: Timer,
    systems: Vec<&'a dyn Fn(&World, Duration, Instant)>,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game {
            world: Default::default(),
            timer: Default::default(),
            systems: vec![],
        }
    }
    fn run(&mut self) -> Scene {
        self.timer.tick();
        for system in self.systems.iter() {
            system(&self.world, self.timer.elapsed(), self.timer.now())
        }
        self.build_scene()
    }
    pub fn spawn(&mut self, components: impl DynamicBundle) -> Entity {
        self.world.spawn(components)
    }
    pub fn spawn_batch<I>(&mut self, iter: I) -> SpawnBatchIter<'_, I::IntoIter>
    where
        I: IntoIterator,
        I::Item: Bundle,
    {
        self.world.spawn_batch(iter)
    }

    pub fn add_system(&mut self, system: &'a dyn Fn(&World, Duration, Instant)) {
        self.systems.push(system)
    }
    fn build_scene(&mut self) -> Scene {
        let mut sprites: Vec<(SpriteId, InstanceRaw)> = vec![];

        for (_, (pos, rot, scale, sprite)) in &mut self
            .world
            .query::<(&Position, &Rotation, &Scale, &Sprite)>()
        {
            let instance_raw = InstanceRaw::from(Instance {
                position: pos.0,
                rotation: rot.0,
                scale: Vec3::splat(scale.0 as f32),
                frame_id: sprite.frame_id,
            });

            sprites.push((sprite.id, instance_raw))
        }

        let mut colliders: Vec<InstanceRaw> = vec![];

        for (_, (pos, rot, collider)) in &mut self
            .world
            .query_mut::<(&Position, &Rotation, &Collider)>()
            .into_iter()
        {
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

        let mut q = self.world.query::<(&ActiveCamera, &ParallaxCamera)>();

        let (_, (_, cam)) = q.iter().next().expect("No camera defined");

        Scene {
            sprite_instances: sprites,
            camera_uniform: cam.generate_matrix(),
            hitbox_instances: colliders,
        }
    }
    fn capture_input(&self, event: winit::event::WindowEvent) {
        let mut q = self.world.query::<&mut KeyboardInput>();
        for (_, mut key) in q.iter() {
            // ignore non keyboard input
            if let WindowEvent::KeyboardInput { input, .. } = event {
                key.0 = Some(input);
            } else {
                key.0 = None;
            }
        }
    }
}

impl<'a> Default for Game<'a> {
    fn default() -> Self {
        Self::new()
    }
}
