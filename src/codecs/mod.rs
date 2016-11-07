pub mod audio;
pub mod video;
pub mod container;

use std::ffi::OsString;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    TooShort,
    TooLong,
    InvalidArg(String, &'static str)
}


impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::TooShort => "Too few arguments",
            Error::TooLong => "Too many arguments",
            Error::InvalidArg(_, _) => "Invalid argument",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::TooShort => None,
            Error::TooLong => None,
            Error::InvalidArg(_, _) => None
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {


        match *self {
            Error::TooShort | Error::TooLong => write!(f, "{}", self.description()),
            Error::InvalidArg(_, detail) => write!(f, "{}: {}", self.description(), detail)
        }
    }
}

pub trait Codec: Sized {
    fn from_args<'a, T: Iterator<Item = &'a str>>(T) -> Result<(Self, T), Error>;
    fn to_ffmpeg_args(&self) -> Vec<OsString>;
    fn to_ffprobe_id(&self) -> (Option<&'static str>, Option<&'static str>);
    fn to_examples() -> Vec<Vec<&'static str>>;
}



pub fn get_container(s: Option<String>) -> Result<container::Codec, Error> {
    match s {
        Some(s) => match try!(container::Codec::from_args(s.split(","))) {
            (codec, mut rest) => {
                match rest.next().is_some() {
                    false => {
                        Ok(codec)
                    }
                    true => {
                        Err(Error::TooLong)
                    }
                }
            }
        },
        None => Ok(container::Codec::default())
    }
}
