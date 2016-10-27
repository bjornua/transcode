use std::path::{Path, PathBuf};
use std::fs::{ReadDir};
#[derive(Debug)]
pub enum PathType {
    Directory(PathBuf),
    File(PathBuf)
}

pub struct PathIterator(Option<ReadDir>);

impl PathIterator {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        PathIterator(
            path.as_ref().read_dir().ok()
        )
    }
}
impl Iterator for PathIterator {
    type Item = PathType;
    fn next(&mut self) -> Option<Self::Item> {
        let PathIterator(ref mut iterator) = *self;
        loop {
            let path = match iterator.as_mut().and_then(|i| i.next()) {
                Some(Ok(entry)) => {
                    entry
                }
                Some(Err(_)) => {
                    continue
                }
                None => {
                    iterator.take();
                    return None;
                }
            }.path();

            return Some(match path.is_dir() {
                true => PathType::Directory(path),
                false => PathType::File(path)
            });
        }
    }
}

pub struct RecursivePathIterator {
    iterator: PathIterator,
    tail: Vec<PathBuf>
}

impl RecursivePathIterator {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        RecursivePathIterator {
            iterator: PathIterator::new(path),
            tail: Vec::new()
        }
    }
}

impl Iterator for RecursivePathIterator {
    type Item = PathType;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut RecursivePathIterator{ ref mut iterator, ref mut tail} = self;

        loop {
            if let Some(path) = iterator.next() {
                if let PathType::Directory(p) = path {
                    tail.push(p);
                    continue;
                }
                return Some(path);
            }
            return tail.pop().map(|path| {
                *iterator = PathIterator::new(&path);
                PathType::Directory(path)
            });
        }
    }
}
