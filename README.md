
# Tiny APNG Encoder for Rust

![Animated Cat](https://gyazo.snca.net/2019/01/17-193109-e35952c2667267664475a8f08e8ab35d.png)


# Example

```
use crate::apng::{Color, Delay, Frame, Meta};
use crate::apng::encoder::Encoder;
use std::fs::File;

// Generate 2x2 Animated PNG (4 frames)
let meta = Meta {
    width: 2,
    height: 2,
    color: Color {
        alpha_channel: false,
        bit_depth: 8,
        grayscale: false,
    },
    frames: 4,
};

// Delay = 2 seconds
let frame = Frame {
    delay: Some(Delay::new(2, 1)),
    ..Default::default()
};

let mut file = File::create("rgb-rotation.png").unwrap();
let mut encoder = Encoder::new(&mut file, &meta).unwrap();

// RED   GREEN
// BLACK BLUE
encoder.write_frame(
    &[
 // (x=0,y=0)            (x=1,y=0)
    0xFF, 0x00, 0x00,    0x00, 0xFF, 0x00,
 // (x=0,y=1)            (x=1,y=1)
    0x00, 0x00, 0x00,    0x00, 0x00, 0xFF,
    ],
    None,
    Some(&frame)).unwrap();
// BLACK RED
// BLUE  GREEN
encoder.write_frame(
    &[
    0x00, 0x00, 0x00,   0xFF, 0x00, 0x00,
    0x00, 0x00, 0xFF,   0x00, 0xFF, 0x00,
    ],
    None,
    Some(&frame)).unwrap();
// BLUE  BLACK
// GREEN RED
encoder.write_frame(
    &[
    0x00, 0x00, 0xFF,   0x00, 0x00, 0x00,
    0x00, 0xFF, 0x00,   0xFF, 0x00, 0x00,
    ],
    None,
    Some(&frame)).unwrap();
// GREEN BLUE
// RED   BLACK
encoder.write_frame(
    &[
    0x00, 0xFF, 0x00,   0x00, 0x00, 0xFF,
    0xFF, 0x00, 0x00,   0x00, 0x00, 0x00,
    ],
    None,
    Some(&frame)).unwrap();
// !!IMPORTANT DONT FORGET!!
encoder.finish().unwrap();
```
