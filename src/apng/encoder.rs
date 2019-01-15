
use std::io::{self, Write};

use byteorder::{BigEndian, WriteBytesExt};
use flate2::Compression;
use flate2::Crc;
use flate2::write::ZlibEncoder;

use super::Meta;



const SIGNATURE: [u8;8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];


pub struct Encoder<'a, F: io::Write> {
    height: u32,
    sequence: u32,
    width: u32,
    writer: &'a mut F,
}


impl<'a, F: io::Write> Encoder<'a, F> {
    pub fn new(writer: &'a mut F, meta: &Meta) -> io::Result<Self> {
        let mut instance = Encoder {
            height: meta.height,
            sequence: 0,
            width: meta.width,
            writer,
        };
        Self::write_signature(&mut instance)?;
        Self::write_image_header(&mut instance, meta)?;
        Self::write_animation_control(&mut instance, meta.frames)?;
        Ok(instance)
    }

    pub fn finish(mut self) -> io::Result<()> {
        let zero: [u8;0] = [];
        self.write_chunk(b"IEND", &zero)
    }

    pub fn write_frame(&mut self, image_data: &[u8], row_stride: usize) -> io::Result<()> {
        if self.sequence == 0 {
            self.write_default_image(image_data, row_stride)
        } else {
            self.write_animation_frame(image_data, row_stride)
        }
    }

    fn next_sequence(&mut self) -> u32 {
        let result = self.sequence;
        self.sequence += 1;
        result
    }

    fn write_animation_frame(&mut self, image_data: &[u8], row_stride: usize) -> io::Result<()> {
        self.write_frame_control()?;

        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        buffer.flush()?;
        let mut e = ZlibEncoder::new(&mut buffer, Compression::best());
        for line in image_data.chunks(row_stride) {
            e.write_all(&[0x00]).unwrap();
            e.write_all(line).unwrap();
        }
        e.finish().unwrap();

        self.write_chunk(b"fdAT", &buffer)?;

        Ok(())
    }

    fn write_animation_control(&mut self, frames: u32) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(frames)?;
        buffer.write_u32::<BigEndian>(0)?;
        self.write_chunk(b"acTL", &buffer)
    }

    fn write_chunk(&mut self, chunk_type: &[u8;4], chunk_data: &[u8]) -> io::Result<()> {
        // Length
        self.writer.write_u32::<BigEndian>(chunk_data.len() as u32)?;
        // Type
        self.writer.write(chunk_type)?;
        // Data
        self.writer.write(chunk_data)?;
        // CRC
        let mut crc = Crc::new();
        crc.update(chunk_type);
        crc.update(chunk_data);
        self.writer.write_u32::<BigEndian>(crc.sum() as u32)
    }

    fn write_default_image(&mut self, image_data: &[u8], row_stride: usize) -> io::Result<()> {
        self.write_frame_control()?;

        let mut buffer = vec![];
        let mut e = ZlibEncoder::new(&mut buffer, Compression::best());
        for line in image_data.chunks(row_stride) {
            e.write_all(&[0x00]).unwrap();
            e.write_all(line).unwrap();
        }
        e.finish().unwrap();

        self.write_chunk(b"IDAT", &buffer)?;

        Ok(())
    }

    fn write_frame_control(&mut self) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(self.next_sequence())?;
        buffer.write_u32::<BigEndian>(self.width)?;
        buffer.write_u32::<BigEndian>(self.height)?;
        buffer.write_u32::<BigEndian>(0)?; // Offset X
        buffer.write_u32::<BigEndian>(0)?; // Offset Y
        buffer.write_u16::<BigEndian>(100)?; // Numerator of delay
        buffer.write_u16::<BigEndian>(1000)?; // Denominator of delay
        buffer.write_all(&[0u8, 0u8])?; // Dispose operator, Blending operator
        self.write_chunk(b"fcTL", &buffer)
    }

    fn write_image_header(&mut self, meta: &Meta) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(meta.width)?;
        buffer.write_u32::<BigEndian>(meta.height)?;
        // ... compression_method, filter_method, interlace_method
        buffer.write(&[meta.bit_depth, meta.color.to_u8(), 0, 0, 0])?;
        self.write_chunk(b"IHDR", &buffer)
    }

    fn write_signature(&mut self) -> io::Result<()> {
        self.writer.write(&SIGNATURE)?;
        Ok(())
    }
}
