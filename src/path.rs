use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{ReadDir};
use std::io;
use std::path::{Path, PathBuf};
use utils::common_prefix;

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

pub fn find_relative(a: &Path, b: &Path) -> PathBuf {
    let a: Vec<_> = a.iter().collect();
    let b: Vec<_> = b.iter().collect();

    let common = common_prefix(a.as_slice(), b.as_slice()).len();

    let dots = b[common..].into_iter().map(|_| OsStr::new(".."));
    let path = a[common..].into_iter().map(|&x| x);

    dots.chain(path).collect()
}

pub fn find_relative_cwd<'a>(a: &'a Path) -> Result<PathBuf, io::Error> {
    Ok(find_relative(a, try!(current_dir()).as_ref()))
}
