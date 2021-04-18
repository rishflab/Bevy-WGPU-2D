use crate::asset::SpriteId;
use crate::renderer::gpu_primitives::{CameraUniform, InstanceRaw};

#[derive(Clone)]
pub struct Scene {
    pub sprite_instances: Vec<(SpriteId, InstanceRaw)>,
    pub camera_uniform: CameraUniform,
    pub hitbox_instances: Vec<InstanceRaw>,
}
