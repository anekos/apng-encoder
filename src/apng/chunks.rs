

#[repr(packed)]
pub struct IHDR {
    pub width: u32,
    pub heiht: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}
