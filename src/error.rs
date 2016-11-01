use args;
use ffmpeg;
use conversion;
use source;
use std::error::Error as StdError;

pub enum Error {
    ArgError(args::Error),
    FFmpegError(ffmpeg::Error),
    ConversionError(conversion::Error),
    SourceError(source::Error),
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

pub fn print_error(k: Error) {
    use self::Error::*;
    println!("\n-------------------- Error --------------------");
    match k {
        ArgError(e) => print_arg_error(e),
        ConversionError(e) => print_conversion_error(e),
        FFmpegError(e) => print_ffmpeg_error(e),
        SourceError(e) => print_source_error(e),
    }
    println!("-----------------------------------------------");
}


fn print_arg_error(kind: args::Error) {
    use args::Error::*;
    println!("Error: Argument failure ({})", kind.description());;
    match kind {
        MissingProgramName => {
            ()
        },
        MissingInputs { program_name } => {
            println!("");
            println!("Usage: {}", args::Args::usage(program_name))
        },
    }
}

fn print_ffmpeg_error(err: ffmpeg::Error) {
    println!("Error: FFmpeg failure ({})", err.description());
    match err {
        ffmpeg::Error::RunError {stdout, stderr} => {
            println!("FFmpeg stderr:\n{}\n\nFFmpeg stdout:\n{}", stderr.trim(), stdout.trim())
        },
        ffmpeg::Error::IO(_) => (),
        ffmpeg::Error::NoStderr => ()
    }
}

fn print_conversion_error(err: conversion::Error) {
    println!("Conversion error");
    println!("{}", err);
}
fn print_source_error(err: source::Error) {
    println!("{}", err);
}
