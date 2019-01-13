
use std::io;
use std::mem;
use std::slice;

use super::chunks::IHDR;
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

    fn write_chunk<T>(&mut self, chunk_type: &[u8;4], chunk: &T) -> io::Result<()> {
        let bytes = mem::size_of::<T>();
        // Length
        self.write_u32(bytes as u32)?;
        // Type
        self.writer.write(chunk_type)?;
        // Data
        let buffer: &[u8] = unsafe { slice::from_raw_parts(chunk as *const T as *const u8, bytes) };
        self.writer.write(buffer)?;
        // CRC
        // TODO self.write_crc32(buffer)?;
        Ok(())
    }

    fn write_image_header(&mut self, meta: &Meta) -> io::Result<()> {
        let chunk = IHDR {
            width: meta.width,
            heiht: meta.heiht,
            bit_depth: meta.bit_depth,
            color_type: meta.color_type as u8,
            compression_method: 0,
            filter_method: 0,
            interlace_method: 0,
        };

        self.write_chunk(b"IHDR", &chunk)
    }

    fn write_signature(&mut self) -> io::Result<()> {
        self.writer.write(&SIGNATURE)?;
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> io::Result<()> {
        let bytes: [u8; 4] = unsafe { mem::transmute(value.to_le()) };
        self.writer.write(&bytes)?;
        Ok(())
    }
}
