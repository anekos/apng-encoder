
use failure::Fail;
use std::io::Error as IOError;


pub type ApngResult<T> = Result<T, ApngError>;



#[derive(Fail, Debug)]
pub enum ApngError {
    #[fail(display = "Write a default image at first")]
    DefaultImageNotAtFirst,
    #[fail(display = "Invalid argument")]
    InvalidArgument,
    #[fail(display = "Invalid color")]
    InvalidColor,
    #[fail(display = "Invalid default image size or offset")]
    InvalidDefaultImageRectangle,
    #[fail(display = "IO error: {}", 0)]
    Io(IOError),
    #[fail(display = "Default image already exists")]
    MulitiDefaultImage,
    #[fail(display = "Not enough frames: expected={}, actual={}", 0, 1)]
    NotEnoughFrames(usize, usize),
    #[fail(display = "Not enough argument")]
    NotEnoughArgument,
    #[fail(display = "Too large image")]
    TooLargeImage,
    #[fail(display = "Too many frames: expected={}, actual={}", 0, 1)]
    TooManyFrames(usize, usize),
    #[fail(display = "Too small image")]
    TooSmallImage,
}

macro_rules! define_error {
    ($source:ty, $kind:tt) => {
        impl From<$source> for ApngError {
            fn from(error: $source) -> ApngError {
                ApngError::$kind(error)
            }
        }
    }
}

define_error!(std::io::Error, Io);
