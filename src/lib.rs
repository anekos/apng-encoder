
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
    use rand::prelude::*;

    use crate::apng::encoder::{Encoder, Filter};
    use crate::apng::{Color, Delay, Frame, Meta};

    #[cfg(feature = "benchmark")]
    use test::Bencher;


    fn load_source(filepath: &str) -> (Meta, Vec<u8>) {
        let source_file = File::open(filepath).unwrap();
        let decoder = PNGDecoder::new(source_file).unwrap();
        let (width, height) = decoder.dimensions();
        let meta = Meta {
            width: width as u32,
            height: height as u32,
            color: Color::RGB(8),
            frames: 0,
            plays: None, // Infinite loop
        };
        (meta, decoder.read_image().unwrap())
    }


    fn load_sources() -> (Meta, Vec<Vec<u8>>) {
        let (mut meta, image_data) = load_source("test-files/1.png");
        let mut result = vec![image_data];
        meta.frames = 4;

        for i in 2 ..= 4 {
            let (_, image_data) = load_source(&format!("test-files/{}.png", i));
            result.push(image_data);
        }

        (meta, result)
    }

    fn generate_png<F: Write>(file: &mut F, sources: &[Vec<u8>], meta: &Meta, filter: Option<Filter>) {
        // Delay = 2 seconds
        let frame = Frame { delay: Some(Delay::new(1, 10)), ..Default::default() };
        let mut encoder = Encoder::create(file, &meta).unwrap();
        for source in sources {
            encoder.write_frame(&source, None, Some(&frame), filter).unwrap();
        }
        encoder.finish().unwrap();
    }

    fn test_generate_png(filename: &str, filter: Option<Filter>) {
        let (meta, sources) = load_sources();
        let _ = create_dir("test-output");
        let mut file = File::create(format!("test-output/{}", filename)).unwrap();
        generate_png(&mut file, &sources, &meta, filter)

    }

    #[cfg(feature = "benchmark")]
    fn bench_generate_png(b: &mut Bencher, filter: Filter) {
        let (meta, sources) = load_sources();
        b.iter(|| {
            let mut file = vec![];
            generate_png(&mut file, &sources, &meta, Some(filter));
        });
    }

    #[test]
    fn test_generate_png_without_filter() {
        test_generate_png("cherenkov-none.png", Some(Filter::None));
    }

    #[test]
    fn test_generate_png_with_sub_filter() {
        test_generate_png("cherenkov-sub.png", Some(Filter::Sub));
    }

    #[test]
    fn test_generate_png_with_up_filter() {
        test_generate_png("cherenkov-up.png", Some(Filter::Up));
    }

    #[test]
    fn test_generate_png_with_average_filter() {
        test_generate_png("cherenkov-average.png", Some(Filter::Average));
    }

    #[test]
    fn test_generate_png_with_paeth_filter() {
        test_generate_png("cherenkov-paeth.png", Some(Filter::Paeth));
    }

    #[test]
    fn test_generate_png_with_inferred_filter() {
        test_generate_png("cherenkov-infer.png", None);
    }

    #[test]
    fn test_generate_shida() {
        const WIDTH: usize = 200;
        const HEIGHT: usize = 200;
        const PX: usize = 2;

        fn w1x(x: f64, y: f64) -> f64 { x * 0.836 + 0.044 * y }
        fn w1y(x: f64, y: f64) -> f64 { x * -0.044 + 0.836 * y + 0.169 }
        fn w2x(x: f64, y: f64) -> f64 { x * -0.141 + 0.302 * y }
        fn w2y(x: f64, y: f64) -> f64 { x * 0.302 + 0.141 * y + 0.127 }
        fn w3x(x: f64, y: f64) -> f64 { x * 0.141 + -0.302 * y }
        fn w3y(x: f64, y: f64) -> f64 { x * 0.302 + 0.141 * y + 0.169 }
        fn w4x(_: f64, _: f64) -> f64 { 0.0 }
        fn w4y(_: f64, y: f64) -> f64 { 0.175337 * y }

        fn f(rng: &mut ThreadRng, buffer: &mut [u8], k: i64, x: f64, y: f64) {
            if 0 <= k {
                f(rng, buffer, k - 1,  w1x(x, y), w1y(x, y));
                if rng.gen::<f64>() <= 0.3 {
                    f(rng, buffer, k - 1, w2x(x, y), w2y(x, y));
                }
                if rng.gen::<f64>() <= 0.3 {
                    f(rng, buffer, k - 1, w3x(x, y), w3y(x, y));
                }
                if rng.gen::<f64>() <= 0.3 {
                    f(rng, buffer, k - 1, w4x(x, y), w4y(x, y));
                }
            } else {
                let xi = (x + 0.5) * 0.98 * WIDTH as f64;
                let yi = (1.0 - y * 0.98) * HEIGHT as f64;
                let (xi, yi) = (xi.floor() as usize, yi.floor() as usize);
                let base = yi * WIDTH * PX + xi * PX;
                for i in 0 .. PX {
                    buffer[base + i] = 0xff;
                }
            }
        }

        let mut rng = rand::thread_rng();

        let mut file = File::create("test-output/shida.png").unwrap();
        let meta = Meta {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            color: Color::Grayscale(16),
            frames: 3,
            plays: None, // Infinite loop
        };
        let frame = Frame { delay: Some(Delay::new(1, 1)), ..Default::default() };
        let mut encoder = Encoder::create(&mut file, &meta).unwrap();
        for i in 0 .. meta.frames {
            let mut buffer = vec![];
            buffer.resize(WIDTH * HEIGHT * PX, 0);
            f(&mut rng, buffer.as_mut_slice(), 10 + i as i64 * 5, 0.0, 0.0);
            encoder.write_frame(&buffer, None, Some(&frame), None).unwrap();
        }
        encoder.finish().unwrap();
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_without_filter(b: &mut Bencher) {
        bench_generate_png(b, Filter::None);
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_with_sub_filter(b: &mut Bencher) {
        bench_generate_png(b, Filter::Sub);
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_with_up_filter(b: &mut Bencher) {
        bench_generate_png(b, Filter::Up);
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_with_average_filter(b: &mut Bencher) {
        bench_generate_png(b, Filter::Average);
    }

    #[bench]#[cfg(feature = "benchmark")]
    fn bench_with_paeth_filter(b: &mut Bencher) {
        bench_generate_png(b, Filter::Paeth);
    }
}
