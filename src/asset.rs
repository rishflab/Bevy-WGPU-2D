use crate::sprite::AnimTimeline;
use image::{GenericImage, RgbaImage};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub type SpriteId = usize;

pub struct SpriteRegistry(Vec<SpriteData>);

impl SpriteRegistry {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, data: SpriteData) -> SpriteId {
        self.0.push(data);
        self.0.len() - 1
    }
}

impl IntoIterator for SpriteRegistry {
    type Item = SpriteData;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct View {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct SpriteData {
    pub id: String,
    pub frames: Vec<RgbaImage>,
}

impl SpriteData {
    pub fn load(id: &str, frames: Vec<&str>) -> Self {
        SpriteData {
            id: id.to_string(),
            frames: frames
                .iter()
                .map(|path| {
                    image::open(path)
                        .expect("valid sprite path provided")
                        .into_rgba8()
                })
                .collect(),
        }
    }

    pub fn load_from_json(id: &str, file: &str) -> (AnimTimeline, SpriteData) {
        let path = Path::new(file);

        let mut file = File::open(&path).unwrap();

        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();

        let deserialized: AnimTimeline = serde_json::from_str(&s).unwrap();

        let mut frames = Vec::new();

        for anim in deserialized.0.iter() {
            let f: Vec<RgbaImage> = anim
                .iter()
                .map(|i| {
                    let mut image = image::open(&i.png).unwrap();
                    image
                        .sub_image(i.view.x, i.view.y, i.view.width, i.view.height)
                        .to_image()
                })
                .collect();
            frames.push(f);
        }

        let sprite_data = SpriteData {
            id: id.to_string(),
            frames: frames.into_iter().flatten().collect(),
        };
        (deserialized, sprite_data)
    }
}

impl Default for SpriteRegistry {
    fn default() -> Self {
        Self::new()
    }
}
