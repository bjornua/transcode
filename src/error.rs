use args;
use ffmpeg;
use conversion;
use source;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ArgError(args::Error),
    FFmpegError(ffmpeg::Error),
    ConversionError(conversion::Error),
    SourceError(source::Error),
    NoSourcesError
}

impl From<ffmpeg::Error> for Error {
    fn from(err: ffmpeg::Error) -> Self {
        Error::FFmpegError(err)
    }
}
impl From<args::Error> for Error {
    fn from(err: args::Error) -> Self {
        Error::ArgError(err)
    }
}

impl From<source::Error> for Error {
    fn from(err: source::Error) -> Self {
        Error::SourceError(err)
    }
}
impl From<conversion::Error> for Error {
    fn from(err: conversion::Error) -> Self {
        Error::ConversionError(err)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::SourceError(_) => "Source error",
            Error::ArgError(_) => "Argument error",
            Error::FFmpegError(_) => "FFmpeg error",
            Error::ConversionError(_) => "Conversion error",
            Error::NoSourcesError => "No sources where found"
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::SourceError(ref e) => Some(e),
            Error::ArgError(ref e) => Some(e),
            Error::FFmpegError(ref e) => Some(e),
            Error::ConversionError(ref e) => Some(e),
            Error::NoSourcesError => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


pub fn stack_printer(e: &StdError) {
    use utils::repeat_str;
    println!("{}", e);

    let mut e: &StdError = e;
    let mut level = 1;

    while let Some(cause) = e.cause() {
        println!("{}→ {}", repeat_str(" ", level * 4), cause);
        e = cause;
        level += 1;
    }
}




// pub fn print_error(k: Error) {
//     use self::Error::*;
//     println!("\n-------------------- Error --------------------");
//     match k {
//         ArgError(e) => print_arg_error(e),
//         ConversionError(e) => print_conversion_error(e),
//         FFmpegError(e) => print_ffmpeg_error(e),
//         SourceError(e) => print_source_error(e),
//         NoSourcesError
//     }
//     println!("-----------------------------------------------");
// }


// fn print_arg_error(kind: args::Error) {
//     use args::Error::*;
//     println!("Error: Argument failure ({})", kind.description());;
//     match kind {
//         MissingProgramName => {
//             ()
//         },
//         MissingInputs { program_name } => {
//             println!("");
//             println!("Usage: {}", args::Args::usage(program_name))
//         },
//     }
// }

// fn print_ffmpeg_error(err: ffmpeg::Error) {
//     println!("Error: FFmpeg failure ({})", err.description());
//     match err {
//         ffmpeg::Error::RunError {stdout, stderr} => {
//             println!("FFmpeg stderr:\n{}\n\nFFmpeg stdout:\n{}", stderr.trim(), stdout.trim())
//         },
//         ffmpeg::Error::IO(_) => (),
//         ffmpeg::Error::NoStderr => ()
//     }
// }

// fn print_conversion_error(err: conversion::Error) {
//     println!("Conversion error");
//     println!("{}", err);
// }
// fn print_source_error(err: source::Error) {
//     println!("{}", err);
// }
