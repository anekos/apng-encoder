use std::fs::{create_dir, File};
use std::io::Write;

use image::ImageDecoder;
use image::png::PNGDecoder;
use rand::prelude::*;

use apng_encoder::apng::encoder::{Encoder, Filter};
use apng_encoder::apng::{Color, Delay, Frame, Meta};

#[cfg(feature = "benchmark")]
use test::Bencher;



const FOUR: [u8;12] = [
    // (x=0,y=0)            (x=1,y=0)
    0xFF, 0x00, 0x00,    0x00, 0xFF, 0x00,
    // (x=0,y=1)            (x=1,y=1)
    0x00, 0x00, 0x00,    0x00, 0x00, 0xFF,
];


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

fn generate_png<F: Write>(file: &mut F, sources: &[Vec<u8>], meta: Meta, filter: Option<Filter>) {
    // Delay = 2 seconds
    let frame = Frame { delay: Some(Delay::new(1, 10)), ..Default::default() };
    let mut encoder = Encoder::create(file, meta).unwrap();
    for source in sources {
        encoder.write_frame(&source, Some(&frame), filter, None).unwrap();
    }
    encoder.finish().unwrap();
}

fn create_file(filename: &str) -> File {
    let _ = create_dir("test-output");
    File::create(format!("test-output/{}", filename)).unwrap()
}

fn test_generate_png(filename: &str, filter: Option<Filter>) {
    let (meta, sources) = load_sources();
    generate_png(&mut create_file(filename), &sources, meta, filter)

}

#[cfg(feature = "benchmark")]
fn bench_generate_png(b: &mut Bencher, filter: Filter) {
    let (meta, sources) = load_sources();
    b.iter(|| {
        let mut file = vec![];
        generate_png(&mut file, &sources, meta.clone(), Some(filter));
    });
}

#[test]#[should_panic(expected="TooManyFrames(1, 2)")]
fn test_many_frames_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    encoder.write_frame(&FOUR, None, None, None).unwrap();
    encoder.write_frame(&FOUR, None, None, None).unwrap();
}

#[test]#[should_panic(expected="NotEnoughFrames(2, 1)")]
fn test_not_enough_frames_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 2, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    encoder.write_frame(&FOUR, None, None, None).unwrap();
    encoder.finish().unwrap();
}

#[test]#[should_panic(expected="TooLargeImage")]
fn test_too_large_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let mut image_data = vec![];
    image_data.resize(1000, 0);
    encoder.write_frame(&image_data, None, None, None).unwrap();
}

#[test]#[should_panic(expected="TooSmallImage")]
fn test_too_small_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    encoder.write_frame(&[0x00], None, None, None).unwrap();
}

#[test]#[should_panic(expected="TooLargeImage")]
fn test_too_large_validation_with_offset_x() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 2, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { x: Some(1), ..Default::default() };
    encoder.write_frame(&FOUR, None, None, None).unwrap();
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]#[should_panic(expected="TooLargeImage")]
fn test_too_large_validation_with_offset_y() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 2, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { y: Some(1), ..Default::default() };
    encoder.write_frame(&FOUR, None, None, None).unwrap();
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]#[should_panic(expected="InvalidDefaultImageRectangle")]
fn test_default_image_offset_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { x: Some(1), ..Default::default() };
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]
fn test_default_image_offset_validation_ok() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { y: Some(0), ..Default::default() };
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]#[should_panic(expected="InvalidDefaultImageRectangle")]
fn test_default_image_size_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { width: Some(1), ..Default::default() };
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]
fn test_default_image_size_validation_ok() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    let frame = Frame { height: Some(2), ..Default::default() };
    encoder.write_frame(&FOUR, Some(&frame), None, None).unwrap();
}

#[test]#[should_panic(expected="InvalidColor")]
fn test_color_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(17), frames: 2, plays: None };
    let _ = Encoder::create(&mut buffer, meta).unwrap();
}

#[test]#[should_panic(expected="DefaultImageNotAtFirst")]
fn test_default_image_position_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    encoder.write_frame(&FOUR, None, None, None).unwrap();
    encoder.write_default_image(&FOUR, None, None).unwrap();
}

#[test]#[should_panic(expected="MulitiDefaultImage")]
fn test_default_image_count_validation() {
    let mut buffer = vec![];
    let meta = Meta { width: 2, height: 2, color: Color::RGB(8), frames: 1, plays: None };
    let mut encoder = Encoder::create(&mut buffer, meta).unwrap();
    encoder.write_default_image(&FOUR, None, None).unwrap();
    encoder.write_default_image(&FOUR, None, None).unwrap();
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
fn test_generate_offset() {
    const WIDTH: u32 = 200;
    const HEIGHT: u32 = 100;

    let mut file = create_file("offset.png");

    let frames = 5;
    let meta = Meta {
        width: WIDTH,
        height: HEIGHT,
        color: Color::Grayscale(8),
        frames,
        plays: None, // Infinite loop
    };
    let mut encoder = Encoder::create(&mut file, meta).unwrap();

    let mut buffer = vec![];
    buffer.resize((WIDTH * HEIGHT) as usize, 0);
    let frame = Frame { delay: Some(Delay::new(1, 1)), ..Default::default() };
    encoder.write_frame(&buffer, Some(&frame), None, None).unwrap();

    for i in 1 .. frames {
        let (width, height) = (WIDTH / (frames - 1) * i, HEIGHT / (frames - 1) * i);
        let mut buffer = vec![];
        buffer.resize((height * width) as usize, 0xff);
        let frame = Frame {
            height: Some(height),
            width: Some(width),
            x: Some((WIDTH - width) / 2),
            y: Some((HEIGHT - height) / 2),
            delay: Some(Delay::new(1, 1)),
            ..Default::default()
        };
        encoder.write_frame(&buffer, Some(&frame), None, None).unwrap();
    }
    encoder.finish().unwrap();
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

    let mut file = create_file("shida.png");
    let frames = 3;
    let meta = Meta {
        width: WIDTH as u32,
        height: HEIGHT as u32,
        color: Color::Grayscale(16),
        frames,
        plays: None, // Infinite loop
    };
    let frame = Frame { delay: Some(Delay::new(1, 1)), ..Default::default() };
    let mut encoder = Encoder::create(&mut file, meta).unwrap();
    for i in 0 .. frames {
        let mut buffer = vec![];
        buffer.resize(WIDTH * HEIGHT * PX, 0);
        f(&mut rng, buffer.as_mut_slice(), 10 + i as i64 * 5, 0.0, 0.0);
        encoder.write_frame(&buffer, Some(&frame), None, None).unwrap();
    }
    encoder.finish().unwrap();
}

#[test]
fn test_generate_default_image() {
    const WIDTH: u32 = 10;
    const HEIGHT: u32 = 10;

    let mut file = create_file("default-image.png");

    let frames = 3;
    let meta = Meta {
        width: WIDTH,
        height: HEIGHT,
        color: Color::Grayscale(8),
        frames,
        plays: None, // Infinite loop
    };
    let mut encoder = Encoder::create(&mut file, meta).unwrap();

    let mut buffer = vec![];
    buffer.resize((WIDTH * HEIGHT) as usize, 0);
    encoder.write_default_image(&buffer, None, None).unwrap();
    for i in 0 .. frames as usize {
        for (index, it) in buffer.iter_mut().enumerate() {
            if index % 3 == i {
                *it = 0xff;
            } else {
                *it = 0x00;
            }
        }
        encoder.write_frame(&buffer, None, None, None).unwrap();
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
