
use failure::Fail;



pub type AppResult<T> = Result<T, AppError>;


#[derive(Fail, Debug)]
pub enum AppError {
    #[fail(display = "APNG Error: {}", 0)]
    Apng(apng_encoder::ApngError),
    #[fail(display = "Image error: {}", 0)]
    Image(image::ImageError),
    #[fail(display = "Not a integer: {}", 0)]
    Int(std::num::ParseIntError),
    #[fail(display = "Intermingling color type")]
    InterminglingColorType,
    #[fail(display = "IO error: {}", 0)]
    Io(std::io::Error),
    #[fail(display = "Not enough argument")]
    NotEnoughArgument,
    #[fail(display = "Unsupport color type")]
    UnsupportedColor,
}

macro_rules! define_error {
    ($source:ty, $kind:ident) => {
        impl From<$source> for AppError {
            fn from(error: $source) -> AppError {
                AppError::$kind(error)
            }
        }
    }
}

define_error!(std::io::Error, Io);
define_error!(std::num::ParseIntError, Int);
define_error!(image::ImageError, Image);
define_error!(apng_encoder::ApngError, Apng);
