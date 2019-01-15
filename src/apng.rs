

pub mod encoder;



pub struct Meta {
    pub bit_depth: u8,
    pub color: Color,
    pub frames: u32,
    pub height: u32,
    pub width: u32,
}

#[derive(Default)]
pub struct Color {
    pub palette: bool,
    pub grayscale: bool,
    pub alpha_channel: bool,
}


impl Color {
    pub fn to_u8(&self) -> u8 {
        let mut result = 0;
        if self.palette {
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
