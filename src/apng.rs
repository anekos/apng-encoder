

pub mod encoder;



#[derive(Clone, Copy)]
pub enum ColorType {
    // Grayscale = 0,      // 1,2,4,8,16  Each pixel is a grayscale sample.
    RGB = 2,            // 8,16
    // Indexed = 3,        // 1,2,4,8
    // GrayscaleAlpha = 4, // 8,16
    // RGBAlpha = 6,       // 8,16
}


#[repr(packed)]
pub struct Meta {
    pub width: u32,
    pub heiht: u32,
    pub bit_depth: u8,
    pub color_type: ColorType,
}
