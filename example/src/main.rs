
use std::env;
use std::fs::File;
use std::io::{stdout, BufWriter, Read};
use std::process::exit;

use failure::Fail;
use image::GenericImageView;

use apng_encoder::apng::{Color, Delay, Frame, Meta};
use apng_encoder::apng::encoder::Encoder;

mod errors;

use crate::errors::{AppResult, AppError};



#[derive(Debug, Default, Clone)]
struct EntryParameter {
    delay: Option<Delay>,
    rect: Rectangle,
}

#[derive(Debug, Clone)]
struct Entry {
    filepath: String,
    parameter: EntryParameter,
}

#[derive(Debug, Default)]
struct Setting {
    plays: u32,
    entries: Vec<Entry>,
}

struct Image {
    color: Color,
    data: Vec<u8>,
    height: u32,
    width: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Rectangle {
    x: Option<u32>,
    y: Option<u32>,
    height: Option<u32>,
    width: Option<u32>,
}


fn main() {
    if let Err(err) = app() {
        let mut fail: &Fail = &err;
        let mut message = err.to_string();
        while let Some(cause) = fail.cause() {
            message.push_str(&format!("\n\tcaused by: {}", cause));
            fail = cause;
        }
        eprintln!("Error: {}\n", message);
        print_usage();
        exit(1);
    }
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  apngc {{[--delay|-d <DELAY>] [-x <X>] [-y <Y>] [--width|-w <WIDTH>] [--height|-h <HEIGHT>] <IMAGE_FILE>}}...");
    eprintln!("Delay format:");
    eprintln!("  `1/2` for 0.5 seconds");
    eprintln!("  `3/1` for 3 seconds");
}

fn app() -> AppResult<()> {
    let setting = parse_args()?;

    let out = stdout();
    let mut out = BufWriter::new(out.lock());

    let mut encoder;
    let first_color;

    if let Some(first) = setting.entries.first() {
        let image = load_image(&first.filepath)?;
        let meta = Meta {
            width: image.width,
            height: image.height,
            color: image.color,
            frames: setting.entries.len() as u32,
            plays: Some(setting.plays),
        };
        first_color = image.color;
        encoder = Encoder::create(&mut out, meta)?;
        let frame = make_frame(&first.parameter, image.width, image.height);
        encoder.write_frame(&image.data, Some(&frame), None, None)?;
    } else {
        return Err(AppError::NotEnoughArgument);
    }

    for entry in setting.entries.iter().skip(1) {
        let image = load_image(&entry.filepath)?;
        if first_color != image.color {
            return Err(AppError::InterminglingColorType);
        }
        let frame = make_frame(&entry.parameter, image.width, image.height);
        encoder.write_frame(&image.data, Some(&frame), None, None)?;
    }

    encoder.finish()?;

    Ok(())
}


fn parse_args() -> AppResult<Setting> {
    let mut setting = Setting::default();

    let mut args = env::args().skip(1);
    let mut parameter = EntryParameter::default();

    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args.next() {
        let mut next = || args.next().ok_or(AppError::NotEnoughArgument);

        match &*arg {
            "--help" => {
                print_usage();
                exit(0);
            },
            "-d" | "--delay" =>
                parameter.delay = Some(parse_delay(&next()?)?),
            "-p" | "--plays" =>
                setting.plays = next()?.parse()?,
            "-x" =>
                parameter.rect.x = Some(next()?.parse()?),
            "-y" =>
                parameter.rect.y = Some(next()?.parse()?),
            "-h" | "--height" =>
                parameter.rect.height = Some(next()?.parse()?),
            "-w" | "--width" =>
                parameter.rect.width = Some(next()?.parse()?),
            filepath => {
                let entry = Entry {
                    filepath: filepath.to_owned(),
                    parameter: parameter.clone(),
                };
                setting.entries.push(entry);
            }
        }
    }

    Ok(setting)
}


fn parse_delay(s: &str) -> AppResult<Delay> {
    if let Some(div) = s.find('/') {
        let (numerator, denominator) = s.split_at(div);
        let numerator = numerator.parse()?;
        let denominator = denominator[1..].parse()?;
        return Ok(Delay { numerator, denominator });
    }

    let numerator = s.parse()?;
    Ok(Delay { numerator, denominator: 1000 })
}


fn from_color_type(color_type: image::ColorType) -> AppResult<Color> {
    use image::ColorType::*;

    let result = match color_type {
        Gray(bits) => Color::Grayscale(bits),
        RGB(bits) => Color::RGB(bits),
        GrayA(bits) => Color::GrayscaleA(bits),
        RGBA(bits) => Color::RGBA(bits),
        BGR(_) | BGRA(_) | Palette(_) => return Err(AppError::UnsupportedColor)?,
    };

    Ok(result)
}


fn load_image(filepath: &str) -> AppResult<Image> {
    let mut file = File::open(&filepath)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    let image = image::load_from_memory(&buffer)?;
    let (width, height) = image.dimensions();
    let color = from_color_type(image.color())?;
    Ok(Image { width, color, data: image.raw_pixels(), height})
}


fn make_frame(param: &EntryParameter, width: u32, height: u32) -> Frame {
    Frame {
        delay: param.delay,
        width: Some(param.rect.width.unwrap_or(width)),
        height: Some(param.rect.width.unwrap_or(height)),
        ..Default::default()
    }
}
