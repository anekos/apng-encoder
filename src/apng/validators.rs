

use super::Color;
use super::errors::{ApngResult, ErrorKind};



pub fn validate_color(color: &Color) -> ApngResult<()> {
    use self::Color::*;

    match color {
        Grayscale(b) if [1, 2, 4, 8, 16].contains(&b) => (),
        GrayscaleA(b) | RGB(b) | RGBA(b) if [8, 16].contains(b) => (),
        _ => return Err(ErrorKind::InvalidColor)?,
    };

    Ok(())
}
