use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error("Unexpected end of input")]
    Eof,

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("Invalid UTF-8 string")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),

    #[error("Unsupported type")]
    UnsupportedType,

    #[error("Invalid type")]
    InvalidType,

    #[error("Invalid date/time value")]
    InvalidDateTime,
}

impl serde::de::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl serde::ser::Error for Error {
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}
