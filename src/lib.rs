
#![cfg_attr(feature = "benchmark", allow(unstable_features))]
#![cfg_attr(feature = "benchmark", feature(test))]

#[cfg(feature = "benchmark")]
extern crate test;

pub mod apng;



#[cfg(test)]
mod tests {
    use std::fs::{create_dir, File};
    use std::io::Write;

    use image::ImageDecoder;
    use image::png::PNGDecoder;

    use crate::apng::encoder::{Encoder, Filter};
    use crate::apng::{Color, Delay, Frame, Meta};

    #[cfg(feature = "benchmark")]
    use test::Bencher;


    fn load_sources() -> Vec<Vec<u8>> {
        let mut result = vec![];
        for i in 1 ..= 4 {
            let source_file = File::open(format!("test-files/{}.png", i)).unwrap();
            let decoder = PNGDecoder::new(source_file).unwrap();
            result.push(decoder.read_image().unwrap());
        }
        result
    }

    fn generate_png<F: Write>(file: &mut F, sources: &[Vec<u8>], filter: Option<Filter>) {
        // Generate 2x2 Animated PNG (4 frames)
        let meta = Meta {
            width: 716,
            height: 660,
            color: Color {
                alpha_channel: false,
                bit_depth: 8,
                grayscale: false,
            },
            frames: sources.len() as u32,
            plays: None, // Infinite loop
        };

        // Delay = 2 seconds
        let frame = Frame {
            delay: Some(Delay::new(1, 10)),
            ..Default::default()
        };

        let mut encoder = Encoder::create(file, &meta).unwrap();

        for source in sources {
            encoder.write_frame(&source, None, Some(&frame), filter).unwrap();
        }

        encoder.finish().unwrap();
    }

    #[test]
    fn test_generate_png() {
        let sources = load_sources();
        let _ = create_dir("test-output");
        let mut file = File::create("test-output/cherenkov.png").unwrap();
        generate_png(&mut file, &sources, None)
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_with_up_filter(b: &mut Bencher) {
        let sources = load_sources();
        b.iter(|| {
            let mut file = vec![];
            generate_png(&mut file, &sources, Some(Filter::Up));
        });
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_without_filter(b: &mut Bencher) {
        let sources = load_sources();
        b.iter(|| {
            let mut file = vec![];
            generate_png(&mut file, &sources, Some(Filter::None));
        });
    }
}
