

mod apng;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::apng;
        use std::fs::File;

        let color = apng::Color {
            alpha_channel: false,
            grayscale: false,
            pallete: false,
        };

        let meta = apng::Meta {
            width: 2,
            heiht: 2,
            bit_depth: 8,
            color,
        };

        let mut file = File::create("something.png").unwrap();
        let mut encoder = apng::encoder::Encoder::new(&mut file, &meta).unwrap();
        encoder.write_frame(
            &[
            0xff, 0x00, 0x00, 0x00, 0xff, 0x00,
            0x00, 0x00, 0xff, 0x00, 0x00, 0x00,
            ],
            6).unwrap();
        encoder.finish().unwrap();
        // encoder.write(&[]).unwrap();
    }
}
