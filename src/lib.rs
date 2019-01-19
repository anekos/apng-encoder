

pub mod apng;



#[cfg(test)]
mod tests {
    #[test]
    fn generate_4frames() {
        use std::fs::{create_dir, File};
        use image::png::PNGDecoder;
        use image::ImageDecoder;
        use crate::apng::{Color, Delay, Frame, Meta};
        use crate::apng::encoder::{Encoder, Filter};


        // Generate 2x2 Animated PNG (4 frames)
        let meta = Meta {
            width: 716,
            height: 660,
            color: Color {
                alpha_channel: false,
                bit_depth: 8,
                grayscale: false,
            },
            frames: 4,
            plays: None, // Infinite loop
        };

        let filter = Some(Filter::Up);

        // Delay = 2 seconds
        let frame = Frame {
            delay: Some(Delay::new(1, 10)),
            ..Default::default()
        };

        let mut file = File::create("test-output/cherenkov.png").unwrap();
        let mut encoder = Encoder::create(&mut file, &meta).unwrap();

        for i in 1 ..= 4 {
            let source_file = File::open(format!("test-files/{}.png", i)).unwrap();
            let _ = create_dir("test-output");
            let decoder = PNGDecoder::new(source_file).unwrap();
            let image = decoder.read_image().unwrap();
            encoder.write_frame(&image, None, Some(&frame), filter).unwrap();
        }

        encoder.finish().unwrap();
    }
}
