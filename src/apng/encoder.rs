
use std::io::{self, Write};

use byteorder::{BigEndian, WriteBytesExt};
use flate2::Compression;
use flate2::Crc;
use flate2::write::DeflateEncoder;

use super::Meta;



const SIGNATURE: [u8;8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];


pub struct Encoder<'a, F: io::Write> {
    writer: &'a mut F,
}


impl<'a, F: io::Write> Encoder<'a, F> {
    pub fn new(writer: &'a mut F, meta: &Meta) -> io::Result<Self> {
        let mut instance = Encoder { writer };
        Self::write_signature(&mut instance)?;
        Self::write_image_header(&mut instance, meta)?;
        Ok(instance)
    }

    pub fn write_frame(&mut self, image_data: &[u8]) -> io::Result<()> {
        let mut buffer = vec![];
        let mut e = DeflateEncoder::new(&mut buffer, Compression::none());
        e.write_all(image_data).unwrap();
        e.finish().unwrap();
        self.write_chunk(b"IDAT", &buffer)?;
        Ok(())
    }

    pub fn finish(mut self) -> io::Result<()> {
        let zero: [u8;0] = [];
        self.write_chunk(b"IEND", &zero)
    }

    fn write_chunk(&mut self, chunk_type: &[u8;4], chunk_data: &[u8]) -> io::Result<()> {
        let mut crc = Crc::new();
        // Length
        self.write_u32(chunk_data.len() as u32)?;
        // Type
        self.writer.write(chunk_type)?;
        // Data
        self.writer.write(chunk_data)?;
        // CRC
        crc.update(chunk_type);
        crc.update(chunk_data);
        self.write_u32(crc.sum())?;
        Ok(())
    }

    fn write_image_header(&mut self, meta: &Meta) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(meta.width)?;
        buffer.write_u32::<BigEndian>(meta.heiht)?;
        // ... compression_method, filter_method, interlace_method
        buffer.write(&[meta.bit_depth, meta.color_type as u8, 0, 0, 0])?;
        self.write_chunk(b"IHDR", &buffer)
    }

    fn write_signature(&mut self) -> io::Result<()> {
        self.writer.write(&SIGNATURE)?;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        let mut buffer = vec![];
        buffer.write_u32::<BigEndian>(value).unwrap();
        self.writer.write(&buffer)?;
        Ok(())
    }
}
