use ffprobe;
use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::iter::IntoIterator;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    FFProbeError {
        path: PathBuf,
        error: ffprobe::Error,
    },
    PathError { path: PathBuf, error: io::Error },
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FFProbeError { ref error, .. } => error.description(),
            Error::PathError { .. } => "Could not expand path",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::FFProbeError { ref error, .. } => Some(error),
            Error::PathError { ref error, .. } => Some(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PathError { ref path, .. } => {
                write!(f,
                       "{desc}: {path:?}",
                       desc = self.description(),
                       path = path)
            }
            Error::FFProbeError { .. } => write!(f, "{}", self.description()),
        }
    }
}

type SourceOk<T> = Result<T, Error>;

type SourceResult = SourceOk<Source>;

#[derive(Debug, Clone)]
pub struct Source {
    pub path: PathBuf,
    pub ffprobe: ffprobe::FFProbe,
}

impl Ord for Source {
    fn cmp(&self, other: &Source) -> Ordering {
        self.path.cmp(&other.path)
    }
}
impl Eq for Source {}
impl PartialOrd for Source {
    fn partial_cmp(&self, other: &Source) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Source {
    fn eq(&self, other: &Source) -> bool {
        self.path == other.path
    }
}

#[derive(Debug)]
pub struct Sources(Vec<Source>);

impl IntoIterator for Sources {
    type Item = Source;
    type IntoIter = ::std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Deref for Sources {
    type Target = [Source];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Sources {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Sources {
    pub fn from_paths<T, U>(paths: T) -> Result<(Self, Vec<Error>), Error>
        where T: IntoIterator<Item = U>,
              PathBuf: From<U>
    {
        let paths: Result<Vec<_>, Error> = paths.into_iter()
            .map(PathBuf::from)
            .map(canonicalize)
            .collect();

        let mut expanded_paths: Vec<PathBuf> = try!(paths)
            .into_iter()
            .flat_map(expand_path)
            .collect();

        expanded_paths.sort();
        expanded_paths.dedup();

        let sources = expanded_paths.into_iter().map(|path| {
            ffprobe_it(&path).map(|probe| {
                Source {
                    path: path,
                    ffprobe: probe,
                }
            })
        });

        let (good, ffprobe_fail): (Vec<_>, Vec<_>) = sources.partition(|x| x.is_ok());

        let good = good.into_iter().filter_map(|x| x.ok());
        let ffprobe_fail = ffprobe_fail.into_iter().filter_map(|x| x.err());

        Ok((Sources(good.collect()), ffprobe_fail.collect()))
    }
}

fn canonicalize(path: PathBuf) -> Result<PathBuf, Error> {
    match path.canonicalize() {
        Err(e) => {
            Err(Error::PathError {
                error: e,
                path: path,
            })
        }
        Ok(p) => Ok(p),
    }
}

fn expand_path(path: PathBuf) -> Vec<PathBuf> {
    use path::{RecursivePathIterator, PathType};
    let paths: Vec<PathBuf> = match path.is_dir() {
        false => vec![path],
        true => {
            RecursivePathIterator::new(path)
                .filter_map(|x| {
                    match x {
                        PathType::Directory(_) => None,
                        PathType::File(p) => Some(p),
                    }
                })
                .collect()
        }
    };
    return paths;
}

fn ffprobe_it(path: &PathBuf) -> SourceOk<ffprobe::FFProbe> {
    use self::Error::FFProbeError;

    let res = ffprobe::ffprobe(path);
    match res {
        Err(e) => {
            Err(FFProbeError {
                path: path.to_owned(),
                error: e,
            })
        }
        Ok(r) => Ok(r),
    }
}
