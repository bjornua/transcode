use std::io::{Read, self};
use regex::{Regex, self};
use std::str::{self};

pub struct RegexReadIterator<T: Read> {
    regex: Regex,
    pub buffer: Vec<u8>,
    cursor: usize,
    reader: T,
    closed: bool
}

impl<T: Read> RegexReadIterator<T> {
    pub fn new(regex: &str, reader: T) -> Result<Self, regex::Error> {
        let regex = try!(Regex::new(regex));
        Ok(RegexReadIterator {
            regex: regex,
            reader: reader,
            buffer: Vec::new(),
            cursor: 0,
            closed: false
        })
    }
}
impl<T: Read> Iterator for RegexReadIterator<T> {
    type Item = Result<Vec<Option<String>>, io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.closed {
            return None
        }
        loop {
            {
                let slice = self.buffer.as_slice();
                let m = str::from_utf8(slice).ok()
                    .and_then(|d| self.regex.captures(&d[self.cursor..]))
                    .and_then(|c| c.pos(0).map(|(_, p)| (c, p)));

                if let Some((c, end)) = m {
                    self.cursor += end;
                    let results = c.iter().map(|x| x.map(|x| x.to_owned()));
                    return Some(Ok(results.collect()))
                }
            }
            let mut stack_buffer = [0; 2048];
            match self.reader.read(&mut stack_buffer) {
                Ok(0) => {
                    self.closed = true;
                    return None
                },
                Ok(n) => {
                    self.buffer.extend_from_slice(&mut stack_buffer[0..n]);
                },
                Err(e) => {
                    self.closed = true;
                    return Some(Err(e))
                }
            }
        }
    }
}
