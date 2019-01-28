
use std::env;
use std::fs::File;
use std::io::{stdout, BufWriter};
use std::process::exit;

use failure::Fail;
use image::ImageDecoder;
use image::png::PNGDecoder;

use apng_encoder::apng::{Color, Delay, Frame, Meta};
use apng_encoder::apng::errors::{ApngResult};
use apng_encoder::apng::encoder::Encoder;

mod errors;

use crate::errors::{AppResult, ErrorKind};



#[derive(Debug, Default, Clone)]
struct EntryParameter {
    delay: Option<Delay>,
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


fn main() {
    if let Err(err) = app() {
        let mut fail: &Fail = &err;
        let mut message = err.to_string();

        while let Some(cause) = fail.cause() {
            message.push_str(&format!("\n\tcaused by: {}", cause));
            fail = cause;
        }

        eprintln!("Error: {}", message);

        exit(1);
    }
}

fn app() -> AppResult<()> {
    let mut setting = parse_args()?;

    let out = stdout();
    let mut out = BufWriter::new(out.lock());

    let mut encoder;

    if let Some(first) = setting.entries.pop() {
        let file = File::open(&first.filepath)?;
        let decoder = PNGDecoder::new(file)?;
        let (width, height) = decoder.dimensions();
        let meta = Meta {
            width: width as u32,
            height: height as u32,
            color: from_color_type(&decoder.colortype())?,
            frames: setting.entries.len() as u32 + 1,
            plays: Some(setting.plays),
        };
        encoder = Encoder::create(&mut out, meta)?;
        let image_data: Vec<u8> = decoder.read_image()?;
        encoder.write_frame(&image_data, Some(&first.parameter.to_frame()), None, None)?;
    } else {
        return Err(ErrorKind::NotEnoughArgument)?;
    }

    for entry in setting.entries {
        let file = File::open(&entry.filepath)?;
        let decoder = PNGDecoder::new(file)?;
        encoder.write_frame(&decoder.read_image()?, Some(&entry.parameter.to_frame()), None, None)?;
    }

    encoder.finish()?;

    Ok(())
}


fn parse_args() -> AppResult<Setting> {
    let mut setting = Setting::default();

    let mut args = env::args().skip(1);
    let mut parameter = EntryParameter::default();

    while let Some(arg) = args.next() {
        let mut next = || args.next().ok_or(ErrorKind::NotEnoughArgument);

        match &*arg {
            "-d" | "--delay" =>
                parameter.delay = Some(parse_delay(&next()?)?),
            "-p" | "--plays" =>
                setting.plays = next()?.parse()?,
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


fn parse_delay(s: &str) -> ApngResult<Delay> {
    if let Some(div) = s.find('/') {
        let (numerator, denominator) = s.split_at(div);
        let numerator = numerator.parse()?;
        let denominator = denominator[1..].parse()?;
        return Ok(Delay { numerator, denominator });
    }

    let numerator = s.parse()?;
    Ok(Delay { numerator, denominator: 1000 })
}


fn from_color_type(color_type: &image::ColorType) -> AppResult<Color> {
    use image::ColorType::*;

    let result = match color_type {
        Gray(bits) => Color::Grayscale(*bits),
        RGB(bits) => Color::RGB(*bits),
        GrayA(bits) => Color::GrayscaleA(*bits),
        RGBA(bits) => Color::RGBA(*bits),
        BGR(_) | BGRA(_) | Palette(_) => return Err(ErrorKind::UnsupportedColor)?,
    };

    Ok(result)
}


impl EntryParameter {
    fn to_frame(&self) -> Frame {
        Frame {
            delay: self.delay,
            ..Default::default()
        }
    }
}
