use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyFrame {
    pub index: u8,
    pub time: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimTimeline(pub ArrayVec<KeyFrame, 128>);

impl AnimTimeline {
    pub fn new(key_frame: ArrayVec<KeyFrame, 128>) -> Self {
        AnimTimeline(key_frame)
    }
}

fn main() {
    let mut anim_timeline = ArrayVec::<KeyFrame, 128>::new();

    anim_timeline.push(KeyFrame {
        index: 0,
        time: 0.0,
    });
    anim_timeline.push(KeyFrame {
        index: 1,
        time: 0.1,
    });
    anim_timeline.push(KeyFrame {
        index: 2,
        time: 0.2,
    });
    anim_timeline.push(KeyFrame {
        index: 3,
        time: 0.3,
    });
    anim_timeline.push(KeyFrame {
        index: 4,
        time: 0.4,
    });
    anim_timeline.push(KeyFrame {
        index: 5,
        time: 0.5,
    });
    anim_timeline.push(KeyFrame {
        index: 6,
        time: 0.6,
    });
    anim_timeline.push(KeyFrame {
        index: 7,
        time: 0.7,
    });
    anim_timeline.push(KeyFrame {
        index: 8,
        time: 0.0,
    });
    anim_timeline.push(KeyFrame {
        index: 9,
        time: 0.1,
    });
    anim_timeline.push(KeyFrame {
        index: 10,
        time: 0.2,
    });
    anim_timeline.push(KeyFrame {
        index: 11,
        time: 0.3,
    });
    anim_timeline.push(KeyFrame {
        index: 12,
        time: 0.4,
    });
    anim_timeline.push(KeyFrame {
        index: 13,
        time: 0.5,
    });
    anim_timeline.push(KeyFrame {
        index: 14,
        time: 0.6,
    });
    anim_timeline.push(KeyFrame {
        index: 15,
        time: 0.7,
    });

    // Convert the Point to a JSON string.
    let serialized = serde_json::to_string(&anim_timeline).unwrap();

    // Prints serialized = {"x":1,"y":2}
    println!("serialized = {}", serialized);

    // make file
    let path = Path::new("keyframes.json");
    let display = path.display();

    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(serialized.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => println!("{} contains:\n{}", display, s),
    }

    // Convert the JSON string back to a Point.
    let deserialized: AnimTimeline = serde_json::from_str(&s).unwrap();

    // Prints deserialized = Point { x: 1, y: 2 }
    println!("deserialized = {:?}", deserialized);
}
