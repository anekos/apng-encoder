

pub mod encoder;



#[derive(Clone, Copy)]
pub enum ColorType {
    // Grayscale = 0,      // 1,2,4,8,16  Each pixel is a grayscale sample.
    RGB = 2,            // 8,16
    // Indexed = 3,        // 1,2,4,8
    // GrayscaleAlpha = 4, // 8,16
    // RGBAlpha = 6,       // 8,16
}


pub struct Meta {
    pub width: u32,
    pub heiht: u32,
    pub bit_depth: u8,
    pub color: Color,
}

#[derive(Default)]
pub struct Color {
    pub pallete: bool,
    pub grayscale: bool,
    pub alpha_channel: bool,
}


impl Color {
    pub fn to_u8(&self) -> u8 {
        let mut result = 0;
        if self.pallete {
            result = 0b001;
        }
        if !self.grayscale {
            result |= 0b010;
        }
        if self.alpha_channel {
            result |= 0b100;
        }
        result
    }
}
