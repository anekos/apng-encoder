
use std::default::Default;



pub mod encoder;
pub mod errors;



#[derive(Debug, Clone)]
pub struct Meta {
    pub color: Color,
    /// Number of animation frames
    pub frames: u32,
    pub height: u32,
    /// Number of plays
    pub plays: Option<u32>,
    pub width: u32,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Grayscale(u8),
    GrayscaleA(u8),
    // Palette,
    RGB(u8),
    RGBA(u8),
}

#[derive(Debug, Default, Clone)]
pub struct Frame {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub x: Option<u32>,
    pub y: Option<u32>,
    pub delay: Option<Delay>,
    pub dispose_operator: Option<DisposeOperator>,
    pub blend_operator: Option<BlendOperator>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Delay {
    pub numerator: u16,
    pub denominator: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum DisposeOperator {
    None = 0,
    Background = 1,
    Previous = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum BlendOperator {
    Source = 0,
    Over = 1,
}


impl Color {
    pub fn bit_depth(self) -> u8 {
        use self::Color::*;

        match self {
            Grayscale(b) | GrayscaleA(b) | RGB(b) | RGBA(b) => b,
        }
    }

    pub fn pixel_bytes(self) -> usize {
        use self::Color::*;

        match self {
            Grayscale(16) => 2,
            Grayscale(_) => 1,
            GrayscaleA(16) => 4,
            GrayscaleA(_) => 2,
            RGB(16) => 6,
            RGB(_) => 3,
            RGBA(16) => 8,
            RGBA(_) => 4,
        }
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
