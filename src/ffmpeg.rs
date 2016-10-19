use conversion::Conversion;
use std::process::Command;
use std::process::Stdio;
use std::ffi::OsStr;
use std::thread;
use regexreader::{self, RegexReadIterator};
use std::io::Read;

pub fn ffmpeg(con: &Conversion) {
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
        OsStr::new("/dev/null") // c.target.path
    ]);
    c.stderr(Stdio::piped());
    c.stdout(Stdio::null());
    c.stdin(Stdio::null());

    let mut child = c.spawn().unwrap();
    if let Some(mut stderr) = child.stderr {
        child.stderr = Some(thread::spawn(move || {
            TimeIterator::new(&mut stderr).collect::<Vec<_>>();
            stderr
        }).join().unwrap());
    };

    let _status = child.wait();
}

pub struct TimeIterator<'a, T: Read + 'a>(RegexReadIterator<'a, T>);

impl<'a, T: Read> TimeIterator<'a, T> {
    pub fn new(reader: &'a mut T) -> Self {
        return TimeIterator(RegexReadIterator::new(r"time=([0-9]+):([0-9]+):([0-9]+)\.([0-9]+)", reader).unwrap());
    }
}
impl<'a, T: Read> Iterator for TimeIterator<'a, T> {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut TimeIterator(ref mut regexiter) = self;

        for c in regexiter {
            if let Ok(l) = c {
                println!("{:?}", &l[1..])
            }
        }
        return None
    }

// fn time_reader<T: Read>(reader: &mut T) ->  {
}
