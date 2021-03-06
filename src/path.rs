use std::env::current_dir;
use std::ffi::OsStr;
use std::fs::{ReadDir, create_dir_all};
use std::io;
use std::path::{Path, PathBuf, Component};
use utils::common_prefix;

#[derive(Debug)]
pub enum PathType {
    Directory(PathBuf),
    File(PathBuf),
}

pub struct PathIterator(Option<ReadDir>);

impl PathIterator {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        PathIterator(path.as_ref().read_dir().ok())
    }
}
impl Iterator for PathIterator {
    type Item = PathType;
    fn next(&mut self) -> Option<Self::Item> {
        let PathIterator(ref mut iterator) = *self;
        loop {
            let path = match iterator.as_mut().and_then(|i| i.next()) {
                    Some(Ok(entry)) => entry,
                    Some(Err(_)) => continue,
                    None => {
                        iterator.take();
                        return None;
                    }
                }
                .path();

            return Some(match path.is_dir() {
                true => PathType::Directory(path),
                false => PathType::File(path),
            });
        }
    }
}

pub struct RecursivePathIterator {
    iterator: PathIterator,
    tail: Vec<PathBuf>,
}

impl RecursivePathIterator {
    pub fn new<T: AsRef<Path>>(path: T) -> Self {
        RecursivePathIterator {
            iterator: PathIterator::new(path),
            tail: Vec::new(),
        }
    }
}

impl Iterator for RecursivePathIterator {
    type Item = PathType;
    fn next(&mut self) -> Option<Self::Item> {
        let &mut RecursivePathIterator { ref mut iterator, ref mut tail } = self;

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

pub fn find_relative(target: &Path, base: &Path) -> PathBuf {
    let target: Vec<_> = target.iter().collect();
    let base: Vec<_> = base.iter().collect();

    let common = common_prefix(target.as_slice(), base.as_slice()).len();

    let dots = base[common..].into_iter().map(|_| OsStr::new(".."));
    let path = target[common..].into_iter().map(|&x| x);

    dots.chain(path).collect()
}

pub fn find_relative_cwd<'a>(a: &'a Path) -> Result<PathBuf, io::Error> {
    Ok(find_relative(a, try!(current_dir()).as_ref()))
}

pub fn mkdir_parent(path: &Path) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };
    create_dir_all(parent)
}


// Make absolute path and resolve dots (. and ..)
pub fn normalize(path: &Path) -> Result<PathBuf, io::Error> {
    let path = try!(current_dir()).join(path);

    let mut result: Vec<Component> = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                if let Some(&Component::Normal(_)) = result.last() {
                    result.pop();
                }
            }
            Component::RootDir |
            Component::Prefix(_) |
            Component::Normal(_) => {
                result.push(component);
            }
        }
    }
    Ok(result.into_iter()
        .map(|x| x.as_os_str())
        .collect())
}
