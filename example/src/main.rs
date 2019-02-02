
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{stdout, BufWriter, Read, Write};
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
    offset: Offset,
}

#[derive(Debug, Clone)]
struct Entry {
    filepath: String,
    parameter: EntryParameter,
}

#[derive(Debug, Default)]
struct Setting {
    default_image: Option<String>,
    entries: Vec<Entry>,
    plays: u32,
}

#[derive(Debug, Default)]
struct Parsed {
    output: Option<String>,
    setting: Setting,
}

struct Image {
    color: Color,
    data: Vec<u8>,
    height: u32,
    width: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct Offset {
    x: Option<u32>,
    y: Option<u32>,
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
    eprintln!(include_str!("usage.txt"));
}

fn app() -> AppResult<()> {
    let parsed = parse_args()?;

    if let Some(output) = parsed.output {
        let mut file = OpenOptions::new().write(true).create(true).open(output)?;
        compile(&mut file, &parsed.setting)
    } else {
        let out = stdout();
        let mut out = out.lock();
        compile(&mut out, &parsed.setting)
    }
}


fn compile<T: Write>(out: &mut T, setting: &Setting) -> AppResult<()> {
    let mut out = BufWriter::new(out);

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
        if let Some(default_image) = setting.default_image.as_ref() {
            encoder.write_default_image(&load_image(default_image)?.data, None, None)?;
        }
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


fn parse_args() -> AppResult<Parsed> {
    let mut setting = Setting::default();
    let mut output = None;

    let mut args = env::args().skip(1);
    let mut parameter = EntryParameter::default();

    #[allow(clippy::while_let_on_iterator)]
    while let Some(arg) = args.next() {
        let mut next = || args.next().ok_or(AppError::NotEnoughArgument);

        match &*arg {
            "-h" | "--help" => {
                print_usage();
                exit(0);
            },
            "-d" | "--delay" =>
                parameter.delay = Some(parse_delay(&next()?)?),
            "-p" | "--plays" =>
                setting.plays = next()?.parse()?,
            "-x" =>
                parameter.offset.x = Some(next()?.parse()?),
            "-y" =>
                parameter.offset.y = Some(next()?.parse()?),
            "--default" =>
                setting.default_image = Some(next()?),
            "-o" | "--output" =>
                output = Some(next()?),
            filepath => {
                let entry = Entry {
                    filepath: filepath.to_owned(),
                    parameter: parameter.clone(),
                };
                setting.entries.push(entry);
            }
        }
    }

    Ok(Parsed { setting, output })
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
        width: Some(width),
        height: Some(height),
        ..Default::default()
    }
}
