
use failure::{Backtrace, Context, Fail};
use std::fmt::Display;
use std::fmt;
use std::io::Error as IOError;
use std::num::ParseIntError;

use image::ImageError;

use apng_encoder::apng::errors::{Error as ApngError};

pub type AppResult<T> = Result<T, Error>;



#[derive(Fail, Debug)]
pub enum ErrorKind {
    #[fail(display = "APNG Error")]
    Apng,
    #[fail(display = "Image error")]
    Image,
    #[fail(display = "Invalid option value")]
    InvalidOptionValue,
    #[fail(display = "IO error")]
    Io,
    #[fail(display = "Not enough argument")]
    NotEnoughArgument,
    #[fail(display = "Unsupport color type")]
    UnsupportedColor,
}

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}


impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Error {
        Error {
            inner: error.context(ErrorKind::Io),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Error {
        Error {
            inner: error.context(ErrorKind::InvalidOptionValue),
        }
    }
}

impl From<ApngError> for Error {
    fn from(error: ApngError) -> Error {
        Error {
            inner: error.context(ErrorKind::Apng),
        }
    }
}

impl From<ImageError> for Error {
    fn from(error: ImageError) -> Error {
        Error {
            inner: error.context(ErrorKind::Image),
        }
    }
}
