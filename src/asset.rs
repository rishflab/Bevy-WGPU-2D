use image::{GenericImage, RgbaImage};

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
    pub fn load_from_anim_strips(id: &str, strips: Vec<&str>, view: View) -> Self {
        let frames = strips
            .iter()
            .map(|path| {
                let mut image = image::open(path)
                    .expect("valid sprite path provided")
                    .into_rgba8();
                let frame_count = image.width() / image.height();
                (0..frame_count)
                    .into_iter()
                    .map(|i| {
                        image
                            .sub_image(image.height() * i, 0, image.height(), image.height())
                            .sub_image(view.x, view.y, view.width, view.height)
                            .to_image()
                    })
                    .collect::<Vec<RgbaImage>>()
            })
            .collect::<Vec<Vec<RgbaImage>>>()
            .concat();

        SpriteData {
            id: id.to_string(),
            frames,
        }
    }
}

pub struct View {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
