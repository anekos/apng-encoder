

pub mod apng;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::apng;
        use std::fs::File;

        let color = apng::Color {
            alpha_channel: false,
            grayscale: false,
            palette: false,
        };

        let meta = apng::Meta {
            width: 2,
            height: 2,
            bit_depth: 8,
            color,
            frames: 2,
        };

        let mut file = File::create("something.png").unwrap();
        let mut encoder = apng::encoder::Encoder::new(&mut file, &meta).unwrap();
        encoder.write_frame(
            &[
            0x12, 0x33, 0x21, 0x23, 0x44, 0x32,
            0x34, 0x55, 0x43, 0x45, 0x66, 0x54,
            ],
            6).unwrap();
        encoder.write_frame(
            &[
            0x34, 0x55, 0x43, 0x45, 0x66, 0x54,
            0x12, 0x33, 0x21, 0x23, 0x44, 0x32,
            ],
            6).unwrap();
        encoder.finish().unwrap();
    }
}
