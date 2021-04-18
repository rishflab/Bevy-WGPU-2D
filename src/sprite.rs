use crate::TEXTURE_ARRAY_SIZE;
use arrayvec::ArrayVec;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct KeyFrame {
    pub png: PathBuf,
    pub time: f32,
    pub view: View,
}

#[derive(Deserialize, Debug, Clone)]
pub struct View {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy)]
pub struct Sprite {
    id: usize,
    pub anim_frame_index: u8,
}

impl Sprite {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            anim_frame_index: 0,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnimTimeline(pub Vec<ArrayVec<KeyFrame, TEXTURE_ARRAY_SIZE>>);

impl AnimTimeline {
    /// elapsed = time since animation began (sec)
    /// anim_duration = duration of one animation cycle (sec)
    pub fn current_frame(&self, anim_id: u8, elapsed: f32) -> u8 {
        let last_frame_index = self.0.get(anim_id as usize).unwrap().len() - 1;
        let anim_duration = self
            .0
            .get(anim_id as usize)
            .unwrap()
            .get(last_frame_index)
            .unwrap()
            .time;

        // dt = how far into animation cycle (sec)
        // so we can find what frame should be playing
        let dt = elapsed % anim_duration;
        let mut frame = 0;
        for (i, f) in self.0.get(anim_id as usize).unwrap().iter().enumerate() {
            if dt < f.time {
                frame = i;
                break;
            }
        }

        let start = self.0[0..anim_id as usize].iter().flatten().count();

        (frame + start) as u8
    }
}
