
use std::cmp;
use std::io::{self, Write};

use byteorder::{BigEndian, WriteBytesExt};
use enum_iterator::IntoEnumIterator;
use flate2::Compression;
use flate2::Crc;
use flate2::write::ZlibEncoder;

use super::{Color, Frame, Meta};
use super::errors::{ApngResult, ApngError};


/// APNG Encoder
///
/// # Example
///
/// ```
/// use apng_encoder::{Color, Delay, Frame, Meta};
/// use apng_encoder::Encoder;
/// use std::fs::File;
///
/// // Generate 2x2 Animated PNG (4 frames)
/// let meta = Meta {
///     width: 2,
///     height: 2,
///     color: Color::RGB(8),
///     frames: 4,
///     plays: None, // Infinite loop
/// };
///
/// // Delay = 1/2 (0.5) seconds
/// let frame = Frame {
///     delay: Some(Delay::new(1, 2)),
///     ..Default::default()
/// };
///
/// let mut file = File::create("test-output/2x2.png").unwrap();
/// let mut encoder = Encoder::create(&mut file, meta).unwrap();
///
/// // RED   GREEN
/// // BLACK BLUE
/// encoder.write_frame(
///     &[
///  // (x=0,y=0)            (x=1,y=0)
///     0xFF, 0x00, 0x00,    0x00, 0xFF, 0x00,
///  // (x=0,y=1)            (x=1,y=1)
///     0x00, 0x00, 0x00,    0x00, 0x00, 0xFF,
///     ],
///     Some(&frame),
///     None,
///     None).unwrap();
/// // BLACK RED
/// // BLUE  GREEN
/// encoder.write_frame(
///     &[
///     0x00, 0x00, 0x00,   0xFF, 0x00, 0x00,
///     0x00, 0x00, 0xFF,   0x00, 0xFF, 0x00,
///     ],
///     Some(&frame),
///     None,
///     None).unwrap();
/// // BLUE  BLACK
/// // GREEN RED
/// encoder.write_frame(
///     &[
///     0x00, 0x00, 0xFF,   0x00, 0x00, 0x00,
///     0x00, 0xFF, 0x00,   0xFF, 0x00, 0x00,
///     ],
///     Some(&frame),
///     None,
///     None).unwrap();
/// // GREEN BLUE
/// // RED   BLACK
/// encoder.write_frame(
///     &[
///     0x00, 0xFF, 0x00,   0x00, 0x00, 0xFF,
///     0xFF, 0x00, 0x00,   0x00, 0x00, 0x00,
///     ],
///     Some(&frame),
///     None,
///     None).unwrap();
/// // !!IMPORTANT DONT FORGET!!
/// encoder.finish().unwrap();
/// ```



#[derive(Debug, Eq, PartialEq)]
pub struct Encoder<'a, F: io::Write> {
    default_image: bool,
    meta: Meta,
    sequence: u32,
    writer: &'a mut F,
    written_frames: usize,
}

#[derive(Clone, Copy, Debug, Eq, IntoEnumIterator, PartialEq)]
pub enum Filter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rectangle {
    height: u32,
    modified: bool,
    width: u32,
    x: u32,
    y: u32,
}


impl<'a, F: io::Write> Encoder<'a, F> {
    pub fn create(writer: &'a mut F, meta: Meta) -> ApngResult<Self> {
        validate_color(meta.color)?;
        let mut instance = Encoder {
            default_image: false,
            meta,
            sequence: 0,
            writer,
            written_frames: 0,
        };
        Self::write_signature(&mut instance)?;
        Self::write_image_header(&mut instance)?;
        Self::write_animation_control(&mut instance)?;
        Ok(instance)
    }

    pub fn finish(mut self) -> ApngResult<()> {
        if self.written_frames < self.meta.frames as usize {
            return Err(ApngError::NotEnoughFrames(self.meta.frames as usize, self.written_frames));
        }
        let zero: [u8;0] = [];
        self.write_chunk(*b"IEND", &zero)
    }

    pub fn write_default_image(&mut self, image_data: &[u8], filter: Option<Filter>, row_stride: Option<usize>) -> ApngResult<()> {
        if self.default_image {
            return Err(ApngError::MulitiDefaultImage);
        }
        if 0 < self.sequence {
            return Err(ApngError::DefaultImageNotAtFirst);
        }
        self.default_image = true;
        let rect = self.compute_rect(None);
        let mut buffer = vec![];
        self.make_image_data(image_data, row_stride, &mut buffer, rect, filter)?;
        self.write_chunk(*b"IDAT", &buffer)?;
        Ok(())
    }

    pub fn write_frame(&mut self, image_data: &[u8], frame: Option<&Frame>, filter: Option<Filter>, row_stride: Option<usize>) -> ApngResult<()> {
        self.written_frames += 1;
        if (self.meta.frames as usize) < self.written_frames {
            return Err(ApngError::TooManyFrames(self.meta.frames as usize, self.written_frames));
        }
        if !self.default_image && self.sequence == 0 {
            self.write_animation_frame_with_default(image_data, row_stride, frame, filter)
        } else {
            self.write_animation_frame(image_data, row_stride, frame, filter)
        }
    }

    fn compute_rect(&self, frame: Option<&Frame>) -> Rectangle {
        let width = frame.and_then(|it| it.width).unwrap_or(self.meta.width);
        let height = frame.and_then(|it| it.height).unwrap_or(self.meta.height);
        let x = frame.and_then(|it| it.x).unwrap_or(0);
        let y = frame.and_then(|it| it.y).unwrap_or(0);
        let modified = x != 0 || y != 0 || width != self.meta.width || height != self.meta.height;
        Rectangle { width, height, x, y, modified }
    }

    fn next_sequence(&mut self) -> u32 {
        let result = self.sequence;
        self.sequence += 1;
        result
    }

    fn make_image_data(&mut self, image_data: &[u8], row_stride: Option<usize>, buffer: &mut Vec<u8>, rect: Rectangle, filter: Option<Filter>) -> ApngResult<()> {
        let row_stride = self.compute_row_stride(&image_data, row_stride, rect)?;
        let mut e = ZlibEncoder::new(buffer, Compression::best());
        let pixel_bytes = self.meta.color.pixel_bytes();
        let filter = filter.map(Ok).unwrap_or_else(|| infer_best_filter(image_data, row_stride, pixel_bytes))?;
        filter.apply(image_data, row_stride, pixel_bytes, &mut e)?;
        e.finish()?;
        Ok(())
    }

    fn compute_row_stride(&self, image_data: &[u8], row_stride: Option<usize>, rect: Rectangle) -> ApngResult<usize> {
        let row_stride = row_stride.unwrap_or_else(|| rect.width as usize * self.meta.color.pixel_bytes());
        let data_height = (image_data.len() / row_stride) as u32;
        if self.meta.width < rect.right() || self.meta.height < rect.bottom() || rect.bottom() < data_height{
            return Err(ApngError::TooLargeImage);
        }
        if data_height < rect.height {
            return Err(ApngError::TooSmallImage);
        }
        Ok(row_stride)
    }

    fn write_animation_frame(&mut self, image_data: &[u8], row_stride: Option<usize>, frame: Option<&Frame>, filter: Option<Filter>) -> ApngResult<()> {
        let rect = self.write_frame_control(frame)?;
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        self.make_image_data(image_data, row_stride, &mut buffer, rect, filter)?;
        self.write_chunk(*b"fdAT", &buffer)?;
        Ok(())
    }

    fn write_animation_frame_with_default(&mut self, image_data: &[u8], row_stride: Option<usize>, frame: Option<&Frame>, filter: Option<Filter>) -> ApngResult<()> {
        let rect = self.write_frame_control(frame)?;
        if rect.modified {
            return Err(ApngError::InvalidDefaultImageRectangle);
        }
        let mut buffer = vec![];
        self.make_image_data(image_data, row_stride, &mut buffer, rect, filter)?;
        self.write_chunk(*b"IDAT", &buffer)?;
        Ok(())
    }

    fn write_animation_control(&mut self) -> ApngResult<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.meta.frames)?;
        buffer.write_u32::<BigEndian>(self.meta.plays.unwrap_or(0))?;
        self.write_chunk(*b"acTL", &buffer)
    }

    fn write_chunk(&mut self, chunk_type: [u8;4], chunk_data: &[u8]) -> ApngResult<()> {
        // Length
        self.writer.write_u32::<BigEndian>(chunk_data.len() as u32)?;
        // Type
        self.writer.write_all(&chunk_type)?;
        // Data
        self.writer.write_all(chunk_data)?;
        // CRC
        let mut crc = Crc::new();
        crc.update(&chunk_type);
        crc.update(chunk_data);
        self.writer.write_u32::<BigEndian>(crc.sum() as u32)?;
        Ok(())
    }

    fn write_frame_control(&mut self, frame: Option<&Frame>) -> ApngResult<Rectangle> {
        let rect = self.compute_rect(frame);
        let delay = frame.and_then(|it| it.delay).unwrap_or_default();
        let dispose = frame.and_then(|it| it.dispose_operator).unwrap_or_default() as u8;
        let blend = frame.and_then(|it| it.blend_operator).unwrap_or_default() as u8;

        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        buffer.write_u32::<BigEndian>(rect.width)?;
        buffer.write_u32::<BigEndian>(rect.height)?;
        buffer.write_u32::<BigEndian>(rect.x)?;
        buffer.write_u32::<BigEndian>(rect.y)?;
        buffer.write_u16::<BigEndian>(delay.numerator)?;
        buffer.write_u16::<BigEndian>(delay.denominator)?;
        buffer.write_all(&[dispose, blend])?;
        self.write_chunk(*b"fcTL", &buffer)?;

        Ok(rect)
    }

    fn write_image_header(&mut self) -> ApngResult<()> {
        use super::Color::*;

        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.meta.width)?;
        buffer.write_u32::<BigEndian>(self.meta.height)?;
        // Alpha - Color - Palette
        let color_type = match self.meta.color {
            Grayscale(_) => 0b000,
            GrayscaleA(_) => 0b100,
            RGB(_) => 0b010,
            RGBA(_) => 0b110,
        };
        // ... compression_method, filter_method, interlace_method
        buffer.write_all(&[self.meta.color.bit_depth(), color_type, 0, 0, 0])?;
        self.write_chunk(*b"IHDR", &buffer)
    }

    fn write_signature(&mut self) -> ApngResult<()> {
        self.writer.write_all(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])?;
        Ok(())
    }
}


impl Filter {
    fn apply<E: Write>(self, image_data: &[u8], row_stride: usize, pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
        let f = match self {
            Filter::Average => filter_average,
            Filter::None => filter_none,
            Filter::Paeth => filter_paeth,
            Filter::Sub => filter_sub,
            Filter::Up => filter_up,
        };
        f(image_data, row_stride, pixel_bytes, e)
    }
}


impl Rectangle {
    fn right(&self) -> u32 {
        self.x + self.width
    }

    fn bottom(&self) -> u32 {
        self.y + self.height
    }
}


fn filter_none<E: Write>(image_data: &[u8], row_stride: usize, _pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
    for line in image_data.chunks(row_stride) {
        e.write_all(&[0x00])?;
        e.write_all(line)?;
    }
    Ok(())
}

fn filter_sub<E: Write>(image_data: &[u8], row_stride: usize, pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
    let mut buffer = vec![0; row_stride];

    for line in image_data.chunks(row_stride) {
        e.write_all(&[0x01])?;
        buffer[..pixel_bytes].clone_from_slice(&line[..pixel_bytes]);
        for (i, it) in buffer.iter_mut().enumerate().take(row_stride).skip(pixel_bytes) {
            *it = line[i].wrapping_sub(line[i - pixel_bytes]);
        }
        e.write_all(&buffer)?;
    }

    Ok(())
}

fn filter_up<E: Write>(image_data: &[u8], row_stride: usize, _pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
    let lines: Vec<&[u8]> = image_data.chunks(row_stride).collect();
    let mut buffer = vec![0; row_stride];

    e.write_all(&[0x02])?;
    e.write_all(&lines[0])?;

    for line in lines.windows(2) {
        e.write_all(&[0x02])?;
        for (i, it) in buffer.iter_mut().enumerate().take(row_stride) {
            *it = line[1][i].wrapping_sub(line[0][i]);
        }
        e.write_all(&buffer)?;
    }

    Ok(())
}

fn filter_average<E: Write>(image_data: &[u8], row_stride: usize, pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
    let lines: Vec<&[u8]> = image_data.chunks(row_stride).collect();
    let mut buffer = vec![0; row_stride];

    e.write_all(&[0x03])?;
    buffer[..pixel_bytes].clone_from_slice(&lines[0][..pixel_bytes]);
    for (i, it) in buffer.iter_mut().enumerate().take(row_stride).skip(pixel_bytes) {
        *it = lines[0][i].wrapping_sub(lines[0][i - pixel_bytes] / 2);
    }
    e.write_all(&buffer)?;

    for line in lines.windows(2) {
        e.write_all(&[0x03])?;
        for (i, it) in buffer.iter_mut().enumerate().take(pixel_bytes) {
            *it = line[1][i].wrapping_sub(line[0][i] / 2);
        }
        for (i, it) in buffer.iter_mut().enumerate().take(row_stride).skip(pixel_bytes) {
            let sum = (i16::from(line[1][i - pixel_bytes]) + i16::from(line[0][i])) / 2;
            *it = line[1][i].wrapping_sub(sum as u8);
        }
        e.write_all(&buffer)?;
    }

    Ok(())
}

fn filter_paeth<E: Write>(image_data: &[u8], row_stride: usize, pixel_bytes: usize, e: &mut E) -> ApngResult<()> {
    fn paeth(left: u8, up_left: u8, up: u8) -> u8 {
        let w_left = i16::from(left);
        let w_up = i16::from(up);
        let w_up_left = i16::from(up_left);

        let base = w_left + w_up - w_up_left;
        let d_left = (base - w_left).abs();
        let d_up = (base - w_up).abs();
        let d_up_left = (base - w_up_left).abs();

        if d_left <= d_up && d_left <= d_up_left {
            return left;
        }

        if d_up <= d_up_left {
            return up;
        }

        up_left
    }

    let lines: Vec<&[u8]> = image_data.chunks(row_stride).collect();
    let mut buffer = vec![0; row_stride];

    e.write_all(&[0x04])?;
    buffer[..pixel_bytes].clone_from_slice(&lines[0][..pixel_bytes]);
    for (i, it) in buffer.iter_mut().enumerate().take(row_stride).skip(pixel_bytes) {
        *it = lines[0][i].wrapping_sub(paeth(lines[0][i - pixel_bytes], 0, 0));
    }
    e.write_all(&buffer)?;

    for line in lines.windows(2) {
        e.write_all(&[0x04])?;
        for (i, it) in buffer.iter_mut().enumerate().take(pixel_bytes) {
            *it = line[1][i].wrapping_sub(paeth(0, 0, line[0][i]));
        }
        for (i, it) in buffer.iter_mut().enumerate().take(row_stride).skip(pixel_bytes) {
            *it = line[1][i].wrapping_sub(paeth(line[1][i - pixel_bytes], line[0][i - pixel_bytes], line[0][i]));
        }
        e.write_all(&buffer)?;
    }

    Ok(())
}

fn get_compressed_size(filter: Filter, image_data: &[u8], row_stride: usize, pixel_bytes: usize) -> ApngResult<usize> {
    let mut out = vec![];
    filter.apply(image_data, row_stride, pixel_bytes, &mut out)?;
    Ok(out.len())
}

fn infer_best_filter(image_data: &[u8], row_stride: usize, pixel_bytes: usize) -> ApngResult<Filter> {
    let mut tiny_image_data = vec![];
    let len = image_data.len();
    let lines = len / row_stride;

    if 50 < lines {
        let top_end = row_stride * 10;
        let middle_start = cmp::max(top_end, lines / 2 * row_stride);
        let middle_end = cmp::min(middle_start + 10 * row_stride, len);
        let bottom_start = cmp::max(middle_end, (cmp::max(lines, 10) - 10) * row_stride);

        tiny_image_data.extend_from_slice(&image_data[0 .. top_end]);
        tiny_image_data.extend_from_slice(&image_data[middle_start .. middle_end]);
        tiny_image_data.extend_from_slice(&image_data[bottom_start .. image_data.len()]);
    } else {
        tiny_image_data.extend_from_slice(&image_data[0 .. cmp::min(10, lines) * row_stride]);
    }


    let mut results = vec![];
    for filter in Filter::into_enum_iter() {
        let size = get_compressed_size(filter, &tiny_image_data, row_stride, pixel_bytes)?;
        results.push((filter, size));
    }

    Ok(results.iter().max_by_key(|it| it.1).unwrap().0)
}


fn validate_color(color: Color) -> ApngResult<()> {
    use self::Color::*;

    match color {
        Grayscale(b) if [1, 2, 4, 8, 16].contains(&b) => (),
        GrayscaleA(b) | RGB(b) | RGBA(b) if [8, 16].contains(&b) => (),
        _ => return Err(ApngError::InvalidColor),
    };

    Ok(())
}
