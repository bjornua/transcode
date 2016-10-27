use ffprobe;
use std::path::PathBuf;
use std::iter::{IntoIterator};

#[derive(Debug)]
pub enum ErrorKind {
    FFProbeError {error: ffprobe::Error},
    PathError {error: String}
}
#[derive(Debug)]
pub struct Error {
    pub path: PathBuf,
    pub kind: ErrorKind
}

type SourceResult<T> = Result<T, Error>;


#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub ffprobe: ffprobe::FFProbe
}

pub struct Sources(Vec<Source>);

impl IntoIterator for Sources {
    type Item = Source;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

use std::ops::{Deref, DerefMut};
impl Deref for Sources {
    type Target = [Source];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Sources {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}


pub fn get_many<'a, T, U>(paths: T) -> SourceResult<Sources> where T: IntoIterator<Item=U>, U: Into<PathBuf> {
    let paths = try!(
        paths.into_iter()
        .map(|p| resolve_path(p.into()))
        .collect::<Result<Vec<_>, _>>()
    );

    let ffprobes = try!(
        (&paths).into_iter()
        .map(|p| ffprobe_it(p))
        .collect::<Result<Vec<_>, _>>()
    );


    let sources = paths.into_iter()
        .zip(ffprobes)
        .map(|(s,f)| Source { path: s, ffprobe: f } );

    Ok(Sources(sources.collect()))
}

fn resolve_path(path: PathBuf) -> SourceResult<PathBuf> {
    use self::ErrorKind::{PathError};
    if !path.is_file() {
        return Err(Error{kind: PathError {error: String::from("Is not a file")}, path: path })
    }

    match path.canonicalize() {
        Err(e) => return Err(Error{kind: PathError {error: format!("{}", e)}, path: path }),
        Ok(p) => Ok(p)
    }
}

fn ffprobe_it(path: &PathBuf) -> SourceResult<ffprobe::FFProbe> {
    use self::ErrorKind::*;

    let res = ffprobe::ffprobe(&path.to_str().unwrap());
    match res {
        Err(e) => Err(Error { path: path.to_owned(), kind: FFProbeError {error: e} } ),
        Ok(r) => Ok(r)
    }
}
