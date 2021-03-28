#![feature(in_band_lifetimes)]

use crate::asset::SpriteId;
use crate::input::KeyState;
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
pub mod input;
mod renderer;
pub mod sprite;
mod time;

pub struct Position(pub Vec3);
pub struct Rotation(pub Quat);
pub struct Scale(pub u8);
pub struct Collider(pub parry2d::shape::Cuboid);

pub struct Resources {
    pub now: Instant,
    pub dt: Duration,
    pub key_state: KeyState,
}

pub struct Game<'a> {
    world: World,
    timer: Timer,
    systems: Vec<&'a dyn Fn(&World, Resources)>,
    key_state: KeyState,
}

impl<'a> Game<'a> {
    pub fn new() -> Game<'a> {
        Game {
            world: Default::default(),
            timer: Default::default(),
            systems: vec![],
            key_state: Default::default(),
        }
    }
    fn run(&mut self) -> Scene {
        self.timer.tick();
        for system in self.systems.iter() {
            system(&self.world, Resources {
                dt: self.timer.elapsed(),
                now: self.timer.now(),
                key_state: self.key_state,
            })
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

    pub fn add_system(&mut self, system: &'a dyn Fn(&World, Resources)) {
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
    fn capture_input_event(&mut self, event: winit::event::WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            self.key_state.update(input);
        }
    }
}

impl<'a> Default for Game<'a> {
    fn default() -> Self {
        Self::new()
    }
}
