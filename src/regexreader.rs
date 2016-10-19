use std::io::{Read, self};
use regex::{Regex, self};
use std::str::{self};

pub struct RegexReadIterator<'a, T: Read + 'a> {
    regex: Regex,
    buffer: Vec<u8>,
    cursor: usize,
    reader: &'a mut T,
}

impl<'a, T: Read + 'a> RegexReadIterator<'a, T> {
    pub fn new(regex: &'a str, reader: &'a mut T) -> Result<Self, regex::Error> {
        let regex = try!(Regex::new(regex));
        Ok(RegexReadIterator {
            regex: regex,
            reader: reader,
            buffer: Vec::new(),
            cursor: 0
        })
    }
}
impl<'a, T: Read +  'a> Iterator for RegexReadIterator<'a, T> {
    type Item = Result<Vec<Option<String>>, io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            {
                let slice = self.buffer.as_slice();
                match str::from_utf8(slice) {
                    Err(_) => (), // Utf8Error
                    Ok(decoded) => match self.regex.captures(&decoded[self.cursor..]) {
                        None => (),
                        Some(c) => {
                            match c.pos(0) {
                                None => (),
                                Some((_, end)) => {
                                    self.cursor += end;
                                }
                            };
                            let results = c.iter().map(|x| x.map(|x| x.to_owned()));
                            return Some(Ok(results.collect()))
                        }
                    }
                }
            }
            let mut stack_buffer = [0; 2048];
            match self.reader.read(&mut stack_buffer) {
                Ok(0) => return None,
                Ok(n) => {
                    self.buffer.extend_from_slice(&mut stack_buffer[0..n]);
                },
                Err(e) => return Some(Err(e))
            }
        }
    }
}
