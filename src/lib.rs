

mod apng;



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use crate::apng;
        use std::fs::File;

        let meta = apng::Meta {
            width: 2,
            heiht: 2,
            bit_depth: 8,
            color_type: apng::ColorType::RGB,
        };

        let mut file = File::create("something.png").unwrap();
        let mut encoder = apng::encoder::Encoder::new(&mut file, &meta).unwrap();
        // encoder.write(&[]).unwrap();
    }
}
