use path;
use std::error::Error as StdError;
use std::ffi::{OsString, OsStr};
use std::fmt;
use std::fs;
use std::io;
use std::path::{PathBuf, Path};
use utils;

#[derive(Debug,Clone)]
pub struct Target {
    pub path: PathBuf,
    pub path_tmp: PathBuf,
}


#[derive(Debug)]
pub enum Error {
    Exists { path: PathBuf },
    MkDirError { path: PathBuf, error: io::Error },
    NormalizeError { path: PathBuf, error: io::Error },
    TmpRemoveError { path: PathBuf, error: io::Error },
    TmpRenameError {
        from: PathBuf,
        to: PathBuf,
        error: io::Error,
    },
}
impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Exists { .. } => "Target file exists",
            Error::MkDirError { .. } => "Could not create parent directories for file",
            Error::NormalizeError { .. } => "Could not normalize target path",
            Error::TmpRemoveError { .. } => "Could not remove temporary file",
            Error::TmpRenameError { .. } => "Could not move temporary file to final destination",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Exists { .. } => None,
            Error::MkDirError { ref error, .. } => Some(error),
            Error::NormalizeError { ref error, .. } => Some(error),
            Error::TmpRemoveError { ref error, .. } => Some(error),
            Error::TmpRenameError { ref error, .. } => Some(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Exists { ref path } |
            Error::MkDirError { ref path, .. } |
            Error::NormalizeError { ref path, .. } |
            Error::TmpRemoveError { ref path, .. } => {
                write!(f, "{}: {:?}", self.description(), path)
            }
            Error::TmpRenameError { ref from, ref to, .. } => {
                write!(f, "{}: {:?} -> {:?}", self.description(), from, to)
            }
        }

    }
}

impl Target {
    pub fn new(prefix: &Path, path: &Path, extension: &OsStr) -> Result<Self, Error> {
        let path = prefix.join(path.with_extension(extension));

        let path = match path::normalize(&path) {
            Err(e) => {
                return Err(Error::NormalizeError {
                    path: path,
                    error: e,
                })
            }
            Ok(p) => p,
        };

        if path.exists() {
            return Err(Error::Exists { path: path });
        }

        let path_tmp = path_tmp(&path);

        Ok(Target {
            path: path,
            path_tmp: path_tmp,
        })
    }

    pub fn remove_path_tmp(&self) -> Result<bool, Error> {
        match fs::remove_file(&self.path_tmp) {
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => {
                Err(Error::TmpRemoveError {
                    error: e,
                    path: self.path.clone(),
                })
            }
            Ok(()) => Ok(true),
        }
    }
    pub fn rename_path_tmp(&self) -> Result<(), Error> {
        match fs::rename(&self.path_tmp, &self.path) {
            Err(e) => {
                Err(Error::TmpRenameError {
                    error: e,
                    from: self.path_tmp.clone(),
                    to: self.path.clone(),
                })
            }
            Ok(()) => Ok(()),
        }
    }

    pub fn mkdir_parent(&self) -> Result<(), Error> {
        match path::mkdir_parent(&self.path) {
            Ok(()) => Ok(()),
            Err(e) => {
                Err(Error::MkDirError {
                    path: self.path.clone(),
                    error: e,
                })
            }
        }
    }
}

fn path_tmp(path: &Path) -> PathBuf {
    let hash = OsString::from(utils::hash_to_hex(&path));
    let mut filename = path.file_stem().unwrap_or("".as_ref()).to_os_string();
    filename.push(" - ");
    filename.push(hash);
    if let Some(s) = path.extension() {
        filename.push(".");
        filename.push(s)
    }
    path.with_file_name(filename)
}
