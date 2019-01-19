
use std::io::{self, Write};

use byteorder::{BigEndian, WriteBytesExt};
use flate2::Compression;
use flate2::Crc;
use flate2::write::ZlibEncoder;

use super::{Frame, Meta};


/// APNG Encoder
///
/// # Example
///
/// ```
/// use tiny_apng::apng::{Color, Delay, Frame, Meta};
/// use tiny_apng::apng::encoder::Encoder;
/// use std::fs::File;
///
/// // Generate 2x2 Animated PNG (4 frames)
/// let meta = Meta {
///     width: 2,
///     height: 2,
///     color: Color {
///         alpha_channel: false,
///         bit_depth: 8,
///         grayscale: false,
///     },
///     frames: 4,
///     plays: None, // Infinite loop
/// };
///
/// // Delay = 2 seconds
/// let frame = Frame {
///     delay: Some(Delay::new(2, 1)),
///     ..Default::default()
/// };
///
/// let mut file = File::create("test-output/2x2.png").unwrap();
/// let mut encoder = Encoder::create(&mut file, &meta).unwrap();
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
///     None,
///     Some(&frame),
///     None).unwrap();
/// // BLACK RED
/// // BLUE  GREEN
/// encoder.write_frame(
///     &[
///     0x00, 0x00, 0x00,   0xFF, 0x00, 0x00,
///     0x00, 0x00, 0xFF,   0x00, 0xFF, 0x00,
///     ],
///     None,
///     Some(&frame),
///     None).unwrap();
/// // BLUE  BLACK
/// // GREEN RED
/// encoder.write_frame(
///     &[
///     0x00, 0x00, 0xFF,   0x00, 0x00, 0x00,
///     0x00, 0xFF, 0x00,   0xFF, 0x00, 0x00,
///     ],
///     None,
///     Some(&frame),
///     None).unwrap();
/// // GREEN BLUE
/// // RED   BLACK
/// encoder.write_frame(
///     &[
///     0x00, 0xFF, 0x00,   0x00, 0x00, 0xFF,
///     0xFF, 0x00, 0x00,   0x00, 0x00, 0x00,
///     ],
///     None,
///     Some(&frame),
///     None).unwrap();
/// // !!IMPORTANT DONT FORGET!!
/// encoder.finish().unwrap();
/// ```



pub struct Encoder<'a, F: io::Write> {
    height: u32,
    pixel_size: usize,
    sequence: u32,
    width: u32,
    writer: &'a mut F,
}

#[derive(Clone, Copy)]
pub enum Filter {
    None = 0,
    Sub = 1,
    Up = 2,
    Average = 3,
    Paeth = 4,
}


impl<'a, F: io::Write> Encoder<'a, F> {
    pub fn create(writer: &'a mut F, meta: &Meta) -> io::Result<Self> {
        let mut instance = Encoder {
            height: meta.height,
            pixel_size: meta.color.pixel_size(),
            sequence: 0,
            width: meta.width,
            writer,
        };
        Self::write_signature(&mut instance)?;
        Self::write_image_header(&mut instance, meta)?;
        Self::write_animation_control(&mut instance, meta.frames, meta.plays.unwrap_or(0))?;
        Ok(instance)
    }

    pub fn finish(mut self) -> io::Result<()> {
        let zero: [u8;0] = [];
        self.write_chunk(*b"IEND", &zero)
    }

    pub fn write_frame(&mut self, image_data: &[u8], row_stride: Option<usize>, frame: Option<&Frame>, filter: Option<Filter>) -> io::Result<()> {
        if self.sequence == 0 {
            self.write_default_image(image_data, row_stride, frame, filter)
        } else {
            self.write_animation_frame(image_data, row_stride, frame, filter)
        }
    }

    fn next_sequence(&mut self) -> u32 {
        let result = self.sequence;
        self.sequence += 1;
        result
    }

    fn make_image_data(&mut self, image_data: &[u8], row_stride: Option<usize>, buffer: &mut Vec<u8>, width: u32, filter: Option<Filter>) -> io::Result<()> {
        let row_stride = row_stride.unwrap_or_else(|| width as usize * self.pixel_size);

        let mut e = ZlibEncoder::new(buffer, Compression::best());

        match filter.unwrap_or(Filter::None) {
            Filter::Average => panic!("Not implemented"),
            Filter::None =>
                for line in image_data.chunks(row_stride) {
                    e.write_all(&[0x00])?;
                    e.write_all(line)?;
                },
            Filter::Paeth => panic!("Not implemented"),
            Filter::Sub => panic!("Not implemented"),
            Filter::Up => {
                let lines: Vec<&[u8]> = image_data.chunks(row_stride).collect();
                let mut first = true;
                for line in lines.windows(2) {
                    if first {
                        e.write_all(&[0x02])?;
                        e.write_all(&line[0])?;
                        first = false;
                    }
                    e.write_all(&[0x02])?;
                    let mut buffer = Vec::<u8>::with_capacity(row_stride);
                    for (i, prev) in line[0].iter().enumerate() {
                        buffer.push(line[1][i].wrapping_sub(*prev));
                    }
                    e.write_all(&buffer)?;
                }
            },
        }

        e.finish().unwrap();

        Ok(())
    }

    fn write_animation_frame(&mut self, image_data: &[u8], row_stride: Option<usize>, frame: Option<&Frame>, filter: Option<Filter>) -> io::Result<()> {
        let width = self.write_frame_control(frame)?;
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        self.make_image_data(image_data, row_stride, &mut buffer, width, filter)?;
        self.write_chunk(*b"fdAT", &buffer)?;
        Ok(())
    }

    fn write_animation_control(&mut self, frames: u32, plays: u32) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(frames)?;
        buffer.write_u32::<BigEndian>(plays)?;
        self.write_chunk(*b"acTL", &buffer)
    }

    fn write_chunk(&mut self, chunk_type: [u8;4], chunk_data: &[u8]) -> io::Result<()> {
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
        self.writer.write_u32::<BigEndian>(crc.sum() as u32)
    }

    fn write_default_image(&mut self, image_data: &[u8], row_stride: Option<usize>, frame: Option<&Frame>, filter: Option<Filter>) -> io::Result<()> {
        let width = self.write_frame_control(frame)?;
        let mut buffer = vec![];
        self.make_image_data(image_data, row_stride, &mut buffer, width, filter)?;
        self.write_chunk(*b"IDAT", &buffer)?;
        Ok(())
    }

    fn write_frame_control(&mut self, frame: Option<&Frame>) -> io::Result<u32> {
        let width = frame.and_then(|it| it.width).unwrap_or(self.width);
        let height = frame.and_then(|it| it.height).unwrap_or(self.height);
        let x = frame.and_then(|it| it.x).unwrap_or(0);
        let y = frame.and_then(|it| it.y).unwrap_or(0);
        let delay = frame.and_then(|it| it.delay).unwrap_or_default();
        let dispose = frame.and_then(|it| it.dispose_operator).unwrap_or_default() as u8;
        let blend = frame.and_then(|it| it.blend_operator).unwrap_or_default() as u8;

        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        buffer.write_u32::<BigEndian>(width)?;
        buffer.write_u32::<BigEndian>(height)?;
        buffer.write_u32::<BigEndian>(x)?;
        buffer.write_u32::<BigEndian>(y)?;
        buffer.write_u16::<BigEndian>(delay.numerator)?;
        buffer.write_u16::<BigEndian>(delay.denominator)?;
        buffer.write_all(&[dispose, blend])?;
        self.write_chunk(*b"fcTL", &buffer)?;

        Ok(width)
    }

    fn write_image_header(&mut self, meta: &Meta) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(meta.width)?;
        buffer.write_u32::<BigEndian>(meta.height)?;
        // ... compression_method, filter_method, interlace_method
        buffer.write_all(&[meta.color.bit_depth, meta.color.to_u8(), 0, 0, 0])?;
        self.write_chunk(*b"IHDR", &buffer)
    }

    fn write_signature(&mut self) -> io::Result<()> {
        self.writer.write_all(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])?;
        Ok(())
    }
}
