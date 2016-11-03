use conversion;
use regexreader::RegexReadIterator;
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::io::{self, Read};
use std::process::{self, Command, Stdio};
use std::str;
use path;

#[derive(Debug)]
pub enum Error {
    MkDirError(io::Error),
    SpawnError(io::Error),
    NoStderr,
    OutputError { stdout: String, stderr: String },
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MkDirError(_) => "Could not make directory",
            Error::SpawnError(ref e) if e.kind() == io::ErrorKind::NotFound => "Could not spawn ffmpeg (is ffmpeg installed?)",
            Error::SpawnError(_) => "Could not spawn ffmpeg",
            Error::NoStderr => "There was no stderr in ffmpeg command for some reason",
            Error::OutputError { .. } => "FFmpeg outputted something unexpected",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::MkDirError(ref s) => Some(s),
            Error::SpawnError(ref s) => Some(s),
            Error::NoStderr => None,
            Error::OutputError { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::MkDirError(_) |
            Error::SpawnError(_) |
            Error::NoStderr => write!(f, "{}", self.description()),
            Error::OutputError { ref stdout, ref stderr } => {
                write!(f,
                       "{}\nStdOut:\n{}StdErr:\n{}",
                       self.description(),
                       stdout,
                       stderr)
            }
        }
    }
}

pub struct FFmpegIterator {
    process: process::Child,
    timeiter: TimeIterator<process::ChildStderr>,
    stdout: process::ChildStdout,
    read_once: bool,
}
impl FFmpegIterator {
    pub fn new(con: conversion::Conversion, dry_run: bool) -> Result<Self, Error> {
        let mut c = Command::new("ffmpeg");

        let mut args: Vec<&OsStr> = Vec::new();
        args.extend(&[OsStr::new("-i"),
                      con.source.path.as_ref(),
                      OsStr::new("-f"),
                      OsStr::new("matroska")]);

        if let Some(video) = con.source.ffprobe.video {
            if video.codec == "h264" {
                // Already the right codec, just copy
                args.extend(&[OsStr::new("-c:v"), OsStr::new("copy")]);
            } else {
                args.extend(&[OsStr::new("-c:v"),
                              OsStr::new("libx264"),
                              OsStr::new("-level"),
                              OsStr::new("4.1"),
                              OsStr::new("-preset"),
                              OsStr::new("ultrafast"),
                              OsStr::new("-crf"),
                              OsStr::new("18")]);
            }
        }
        args.extend(&[OsStr::new("-c:a"),
                      OsStr::new("opus"),
                      OsStr::new("-b:a"),
                      OsStr::new("192k")]);

        if dry_run {
            args.extend(&[OsStr::new("-y"), OsStr::new("/dev/null")]);
        } else {
            args.extend(&[con.target.path.as_os_str()]);
        }

        c.args(args.as_slice());

        c.stderr(Stdio::piped());
        c.stdout(Stdio::piped());
        c.stdin(Stdio::null());

        if !dry_run {
            try!(path::mkdir_parent(&con.target.path).map_err(|e| Error::MkDirError(e)));
        }
        let mut child = try!(c.spawn().map_err(|e| Error::SpawnError(e)));
        let stderr = child.stderr.take();
        let stdout = child.stdout.take();
        match (stderr, stdout) {
            (Some(stderr), Some(stdout)) => {
                Ok(FFmpegIterator {
                    process: child,
                    stdout: stdout,
                    timeiter: TimeIterator::new(stderr),
                    read_once: false,
                })
            }
            (_, _) => Err(Error::NoStderr),
        }
    }
}
impl Iterator for FFmpegIterator {
    type Item = Result<f64, Error>;

    fn next(&mut self) -> Option<Self::Item> {

        for x in &mut self.timeiter {
            self.read_once = true;
            return Some(Ok(x));
        }
        let _ = self.process.wait();
        if !self.read_once {
            let buffer_err = String::from_utf8_lossy(&self.timeiter.0.buffer).into_owned();
            let mut buffer_out: Vec<u8> = Vec::new();
            let _ = self.stdout.read_to_end(&mut buffer_out);
            let buffer_out = String::from_utf8_lossy(&buffer_out).into_owned();
            return Some(Err(Error::OutputError {
                stdout: buffer_out,
                stderr: buffer_err,
            }));
        }
        None
    }
}

pub struct TimeIterator<T: Read>(pub RegexReadIterator<T>);

impl<T: Read> TimeIterator<T> {
    pub fn new(reader: T) -> Self {
        const R: &'static str = r"time=([0-9]+):([0-9]+):([0-9]+\.[0-9]+)";
        let regex_iterator = RegexReadIterator::new(R, reader).unwrap();
        return TimeIterator(regex_iterator);
    }
}
impl<'a, T: Read> Iterator for TimeIterator<T> {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut TimeIterator(ref mut regexiter) = self;
        for c in regexiter {
            if let Ok(c) = c {
                let mut i = c.into_iter()
                    .skip(1)
                    .take(3)
                    .filter_map(|x| x.and_then(|x| x.parse::<f64>().ok()));

                if let (Some(h), Some(m), Some(s)) = (i.next(), i.next(), i.next()) {
                    let seconds = h * 3600. + m * 60. + s;
                    return Some(seconds);
                };
            };
        }
        return None;
    }
}
