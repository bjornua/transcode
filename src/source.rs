use check_file;
use ffprobe;
use path;
use std::cmp::Ordering;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::iter::IntoIterator;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    CheckFileError(check_file::Error),
    FFProbeError {
        path: PathBuf,
        error: ffprobe::Error,
    },
    PathError { path: PathBuf, error: io::Error },
    SourceDirectory { path: PathBuf, error: io::Error },
    StraySource { path: PathBuf },
}

impl From<check_file::Error> for Error {
    fn from(err: check_file::Error) -> Self {
        Error::CheckFileError(err)
    }
}


impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::CheckFileError(_) => "Error happened while checking file",
            Error::FFProbeError { .. } => "FFProbe error",
            Error::PathError { .. } => "Could not expand path",
            Error::SourceDirectory { .. } => "Error happened while resolving SOURCE_DIRECTORY",
            Error::StraySource { .. } => "Path cannot be outside SOURCE_DIRECTORY",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::CheckFileError(ref error) => Some(error),
            Error::FFProbeError { ref error, .. } => Some(error),
            Error::PathError { ref error, .. } => Some(error),
            Error::SourceDirectory { ref error, .. } => Some(error),
            Error::StraySource { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::PathError { ref path, .. } |
            Error::FFProbeError { ref path, .. } |
            Error::SourceDirectory { ref path, .. } |
            Error::StraySource { ref path, .. } => {
                write!(f,
                       "{desc}: {path:?}",
                       desc = self.description(),
                       path = path)
            }
            Error::CheckFileError(_) => write!(f, "{}", self.description()),
        }
    }
}

type SourceResult<T> = Result<T, Error>;


#[derive(Debug, Clone)]
pub struct BasedPath {
    pub path: PathBuf,
    pub base: PathBuf,
}
impl BasedPath {
    pub fn relative(&self) -> PathBuf {
        path::find_relative(&self.path, &self.base)
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub path: BasedPath,
    pub ffprobe: ffprobe::FFProbe,
}

impl Ord for Source {
    fn cmp(&self, other: &Source) -> Ordering {
        self.path.path.cmp(&other.path.path)
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
        self.path.path == other.path.path
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
impl Deref for BasedPath {
    type Target = PathBuf;
    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl Sources {
    pub fn from_paths<'a, T, U>(paths: T,
                                base_directory: &'a str)
                                -> SourceResult<(Self, Vec<BasedPath>)>
        where T: IntoIterator<Item = U>,
              U: Into<PathBuf>
    {
        let base_directory = PathBuf::from(base_directory);
        let base_directory = match base_directory.canonicalize() {
            Ok(dir) => dir,
            Err(e) => {
                return Err(Error::SourceDirectory {
                    path: base_directory,
                    error: e,
                })
            }
        };

        let paths: Result<Vec<_>, Error> = paths.into_iter()
            .map(|x| x.into())
            .map(canonicalize)
            .collect();

        let paths = try!(paths);

        if let Some(path) = paths.iter().filter(|&path| !path.starts_with(&base_directory)).next() {
            return Err(Error::StraySource { path: path.clone() });
        }

        let mut expanded_paths: Vec<PathBuf> = paths.into_iter()
            .flat_map(expand_path)
            .collect();

        expanded_paths.sort();
        expanded_paths.dedup();

        // Quick filtering using 'file'
        let (paths, skipped_file) = try!(check_file::check_files(expanded_paths.into_iter()));

        let skipped_file = skipped_file.into_iter().map(|p| {
            BasedPath {
                path: p,
                base: base_directory.clone(),
            }
        });

        let sources: Result<Vec<_>, Error> = paths.into_iter()
            .map(|path| ffprobe_it(&path).map(|probe| (path, probe)))
            .collect();

        let (good, skipped_ffprobe): (Vec<_>, Vec<_>) =
            try!(sources).into_iter().partition(|&(_, ref probe)| probe.is_some());

        let good = good.into_iter().filter_map(|(path, probe)| {
            probe.map(|probe| {
                Source {
                    ffprobe: probe,
                    path: BasedPath {
                        path: path,
                        base: base_directory.clone(),
                    },
                }
            })
        });
        let skipped_ffprobe = skipped_ffprobe.into_iter().map(|(path, _)| {
            BasedPath {
                path: path,
                base: base_directory.clone(),
            }
        });

        Ok((Sources(good.collect()), skipped_file.chain(skipped_ffprobe).collect()))
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

fn ffprobe_it(path: &PathBuf) -> SourceResult<Option<ffprobe::FFProbe>> {
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
