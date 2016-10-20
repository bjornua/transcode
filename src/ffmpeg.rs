use conversion::Conversion;
use std::process::{Command, Stdio, self};
use std::ffi::OsStr;
use regexreader::{RegexReadIterator};
use std::io::{Read, self};

use std::error::{Error as StdError};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    NoStderr
}

impl Error {
    fn get_description(&self) -> &str {
        match *self {
            Error::IO(ref s) => s.description(),
            Error::NoStderr => "There was no stderr in ffmpeg command for some reason"
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.get_description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        self.get_description()
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::IO(ref s) => Some(s),
            Error::NoStderr => None
        }
    }
}


pub struct FFmpegIterator {
    process: process::Child,
    timeiter: TimeIterator<process::ChildStderr>
}

impl FFmpegIterator {
    pub fn new(con: &Conversion) -> Result<Self, Error> {
        let mut c = Command::new("ffmpeg");

        c.args(&[
            OsStr::new("-i"),       OsStr::new(&con.source.path),
            OsStr::new("-f"),       OsStr::new("matroska"),
            OsStr::new("-c:v"),     OsStr::new("libx264"),
            OsStr::new("-level"),   OsStr::new("4.1"),
            OsStr::new("-preset"),  OsStr::new("medium"),
            OsStr::new("-crf"),     OsStr::new("18"),
            OsStr::new("-c:a"),     OsStr::new("opus"),
            OsStr::new("-b:a"),     OsStr::new("192k"),
            OsStr::new("-y"),
            OsStr::new("/dev/null") // con.target.path
        ]);
        c.stderr(Stdio::piped());
        c.stdout(Stdio::null());
        c.stdin(Stdio::null());

        let mut child = c.spawn().unwrap();
        let stderr = child.stderr;
        child.stderr = None;
        match stderr {
            Some( stderr) => {
                Ok(FFmpegIterator {process: child, timeiter: TimeIterator::new(stderr)})
            }
            None => Err(Error::NoStderr)
        }
    }
}
impl Iterator for FFmpegIterator {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        for x in &mut self.timeiter {
            return Some(x)
        }
        let _ = self.process.wait();
        None
    }

}

pub struct TimeIterator<T: Read>(RegexReadIterator<T>);

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
                let mut i = c.into_iter().skip(1).take(3).filter_map(
                    |x| x.and_then(|x| x.parse::<f64>().ok())
                );

                if let (Some(h), Some(m), Some(s)) = (i.next(), i.next(), i.next()) {
                    let seconds = h * 3600. + m * 60. + s;
                    return Some(seconds);
                };
            };
        }
        return None
    }
}
