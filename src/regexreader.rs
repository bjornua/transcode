use std::io::{Read, self};
use regex::{Regex, self};
use std::str;

pub struct RegexReadIterator<'a, T: Read + 'a> {
    regex: Regex,
    buffer: Vec<u8>,
    cursor: usize,
    reader: &'a mut T,
}

impl<'a, T: Read + 'a> RegexReadIterator<'a, T> {
    pub fn new(regex: &str, reader: &'a mut T) -> Result<Self, regex::Error> {
        let regex = try!(Regex::new(regex));
        Ok(RegexReadIterator {
            regex: regex,
            reader: reader,
            buffer: Vec::new(),
            cursor: 0
        })
    }
    fn fill_buffer(&mut self) -> Result<bool, io::Error> {
        let mut stack_buffer = [0; 2048];
        let bytes_read = match self.reader.read(&mut stack_buffer) {
            Ok(n) => n,
            Err(e) => return Err(e)
        };
        self.buffer.extend_from_slice(&mut stack_buffer[0..bytes_read]);
        Ok(bytes_read != 0)
    }
    fn get_next(&'a mut self) -> Option<regex::Captures> {
        if let Ok(decoded) = str::from_utf8(self.buffer.as_slice()) {
            if let Some(c) = self.regex.captures(&decoded[self.cursor..]) {
                if let Some((_, end)) = c.pos(0) {
                    self.cursor += end;
                }
                Some(c);
            }
        }
        return None
    }
}

impl<'a, T: Read + 'a> Iterator for RegexReadIterator<'a, T> {
    type Item = Result<regex::Captures<'a>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_cursor = 0;
        let mut retval: Self::Item;
        loop {
            if let Some(c) = self.get_next() {
                return Some(Ok(c));
            } else {
                match self.fill_buffer() {

                }
            }
        }
        self.cursor += next_cursor;
        return Some(retval)
    }
}
