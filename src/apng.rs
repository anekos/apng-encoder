
use std::default::Default;



pub mod encoder;



pub struct Meta {
    pub color: Color,
    /// Number of animation frames
    pub frames: u32,
    pub height: u32,
    /// Number of plays
    pub plays: Option<u32>,
    pub width: u32,
}

#[derive(Default)]
pub struct Color {
    // TODO pub palette: bool,
    pub bit_depth: u8,
    pub grayscale: bool,
    pub alpha_channel: bool,
}

#[derive(Default)]
pub struct Frame {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub delay: Option<Delay>,
    pub dispose_operator: Option<DisposeOperator>,
    pub blend_operator: Option<BlendOperator>,
}

#[derive(Default, Clone, Copy)]
pub struct Delay {
    pub numerator: u16,
    pub denominator: u16,
}

#[derive(Clone, Copy)]
pub enum DisposeOperator {
    None = 0,
    Background = 1,
    Previous = 2,
}

#[derive(Clone, Copy)]
pub enum BlendOperator {
    Source = 0,
    Over = 1,
}


impl Color {
    pub fn to_u8(&self) -> u8 {
        let mut result = 0;
        // if self.palette {
        //     result = 0b001;
        // }
        if !self.grayscale {
            result |= 0b010;
        }
        if self.alpha_channel {
            result |= 0b100;
        }
        result
    }

    pub fn pixel_size(&self) -> usize {
        let single = if self.bit_depth <= 8 { 1 } else { 2 };
        let alpha_channel = if self.alpha_channel { 1 } else { 0 };
        let base = if self.grayscale { 1 } else { 3 };
        single * (base + alpha_channel)
    }
}


impl Delay {
    pub fn new(numerator: u16, denominator: u16) -> Self {
        Delay { numerator, denominator }
    }
}


impl Default for DisposeOperator {
    fn default() -> Self {
        DisposeOperator::None
    }
}

impl Default for BlendOperator {
    fn default() -> Self {
        BlendOperator::Source
    }
}
