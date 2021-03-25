use crate::renderer::gpu_primitives::{CameraUniform, InstanceRaw};
use std::collections::HashMap;

pub struct Scene {
    pub sprite_instances: HashMap<String, Vec<InstanceRaw>>,
    pub camera_uniform: CameraUniform,
    pub hitbox_instances: HashMap<String, Vec<InstanceRaw>>,
}
