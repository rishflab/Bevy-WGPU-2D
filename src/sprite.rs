use crate::TEXTURE_ARRAY_SIZE;
use arrayvec::ArrayVec;
use serde::Deserialize;
use std::ops::Range;

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct KeyFrame {
    pub index: u8,
    pub time: f32,
}

pub struct Sprite {
    pub id: usize,
    pub frame_id: u8,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnimTimeline(pub ArrayVec<KeyFrame, TEXTURE_ARRAY_SIZE>);

impl AnimTimeline {
    pub fn new(key_frames: impl Iterator<Item = KeyFrame>) -> Self {
        let mut vec = ArrayVec::<KeyFrame, TEXTURE_ARRAY_SIZE>::new();

        for (i, k) in key_frames.enumerate() {
            vec.insert(i, k);
        }

        Self(vec)
    }
    /// elapsed = time since animation began (sec)
    /// anim_duration = duration of one animation cycle (sec)
    pub fn current_frame(&self, strip: Range<usize>, elapsed: f32) -> u8 {
        let last_frame_index = strip.clone().last().unwrap();
        let anim_duration = self.0.get(last_frame_index).unwrap().time;

        // dt = how far into animation cycle (sec)
        // so we can find what frame should be playing
        let dt = elapsed % anim_duration;
        let mut frame = 0;
        for f in self.0[strip].iter() {
            if dt < f.time {
                frame = f.index;
                break;
            }
        }
        frame
    }
}

impl Sprite {
    pub fn new(id: usize) -> Self {
        Self { id, frame_id: 0 }
    }
}
