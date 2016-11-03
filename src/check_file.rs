use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::ffi::OsStr;
use std::io;
use std::string::FromUtf8Error;
use std::error::Error as StdError;
use std::fmt;

const MIME_WHITELIST: &'static [&'static str] = &[
    "audio/mpeg",
    "application/octet-stream",
    "video/mp4",
    "video/mpeg",
    "video/quicktime",
    "video/webm",
    "video/x-flv",
    "video/x-m4v",
    "video/x-matroska",
    "video/x-ms-asf",
    "video/x-msvideo",
    "audio/ogg",
    "audio/x-wav",
];
const MIME_BLACKLIST: &'static [&'static str] = &[
    "application/CDFV2",
    "application/pdf",
    "application/x-7z-compressed",
    "application/x-executable",
    "application/x-sharedlib",
    "image/gif",
    "image/jpeg",
    "image/png",
    "image/x-icon",
    "image/x-ms-bmp",
    "inode/socket",
    "inode/x-empty",
    "regular file, no read permission",
    "text/html",
    "text/plain",
    "text/xml",
    "image/svg+xml",
    "application/zip",
    "application/x-shockwave-flash",
    "image/x-xcf",
];


#[derive(Debug)]
pub enum ErrorKind {
    RunError(io::Error),
    StdOutUTF8Error(FromUtf8Error),
    StdErrUTF8Error(FromUtf8Error),
    ParseError{ stdout: String, stderr: String },
    Unsuccesful {
        stdout: String,
        stderr: String,
        status: Option<i32>,
    },
    UnknownMimeType { mime_type: String },
}

#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    kind: ErrorKind
}
impl Error {
    fn new<'a>(path: &Path, kind: ErrorKind) -> Self {
        return Error {path: path.to_path_buf(), kind: kind}
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.kind {
            ErrorKind::RunError(ref e) if e.kind() == io::ErrorKind::NotFound => "Could not find executable 'file'",
            ErrorKind::RunError(_) => "Could not run 'file'",
            ErrorKind::StdOutUTF8Error(_) => "Could not parse stdout as utf-8",
            ErrorKind::StdErrUTF8Error(_) => "Could not parse stderr as utf-8",
            ErrorKind::ParseError{ .. } => "Could not parse output from file",
            ErrorKind::Unsuccesful { .. } => "File returned non-zero exit code",
            ErrorKind::UnknownMimeType { .. } => "Unknown mime-type"
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match (*self).kind {
            ErrorKind::RunError(ref e)  => Some(e),
            ErrorKind::StdOutUTF8Error(ref e) => Some(e),
            ErrorKind::StdErrUTF8Error(ref e) => Some(e),
            ErrorKind::ParseError { .. } | ErrorKind::Unsuccesful { .. } | ErrorKind::UnknownMimeType { .. } => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::StdOutUTF8Error(_) |
            ErrorKind::StdErrUTF8Error(_) |
            ErrorKind::ParseError{ .. } => write!(f, "{:?}: {}", self.path, self.description()),
            ErrorKind::Unsuccesful { .. } => write!(f, "{:?}: {}", self.path, self.description()),
            ErrorKind::UnknownMimeType { ref mime_type } => write!(f, "{:?}: {} {:?}", self.path, self.description(), mime_type),
            ErrorKind::RunError(_) => write!(f, "{}", self.description())

        }
    }
}


pub fn check_files<'a, T: AsRef<Path>, U: Iterator<Item = T>>(paths: U) -> Result<(Vec<T>, Vec<T>), Error> {
    let paths: Vec<(T, bool)> = try!(paths
        .map(|p| check_file(p.as_ref()).map(|r| (p, r)))
        .collect()
    );

    let (good, bad): (Vec<_>, Vec<_>) = paths.into_iter().partition(|&(_, ref f)| *f);

    let good = good.into_iter().map(|(p, _)| p);
    let bad = bad.into_iter().map(|(p, _)| p);

    Ok((good.collect(), bad.collect()))
}

pub fn check_file<'a>(path: &Path) -> Result<bool, Error> {
    let mut c = Command::new("file");

    c.args(&[OsStr::new("--mime-type"), OsStr::new("-b"), OsStr::new("--dereference"), path.as_os_str()]);

    let result = match c.output() {
        Ok(o) => o,
        Err(e) => return Err(Error::new(path, ErrorKind::RunError(e)))
    };

    let stdout = match String::from_utf8(result.stdout) {
        Ok(s) => s,
        Err(e) => return Err(Error::new(path, ErrorKind::StdOutUTF8Error(e)))
    };
    let stderr = match String::from_utf8(result.stderr) {
        Ok(s) => s,
        Err(e) => return Err(Error::new(path, ErrorKind::StdErrUTF8Error(e)))
    };

    if !result.status.success() {
        return Err(Error::new(path, ErrorKind::Unsuccesful {
            status: result.status.code(),
            stdout: stdout,
            stderr: stderr,
        }))
    }

    let mime_type = stdout.trim();

    if MIME_BLACKLIST.contains(&mime_type) {
        return Ok(false);
    }
    if MIME_WHITELIST.contains(&mime_type) {
        return return Ok(true);
    }

    return Err(Error::new(path, ErrorKind::UnknownMimeType { mime_type: mime_type.to_string() }));

    // println!("Stdout:\n{:#?}\n", stdout.trim());
    // println!("Stderr:\n{:#?}\n", stderr.trim());

}
