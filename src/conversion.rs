use progress::Status;
use source::{Sources, Source};
use std::ffi::OsStr;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use target::Target;
use utils::common_prefix;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Conversion {
    id: u64,
    pub source: Source,
    pub target: Target,
    pub status: Status
}

#[derive(Debug)]
pub enum Error {}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
        }
    }
}
impl fmt::Display for Error { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) } }



impl Conversion {
    pub fn new(id: u64, path: PathBuf, source: Source) -> Self {
        let status = Status::new(source.ffprobe.mpixel());
        let target = Target { path: path };
        Conversion { id: id, target: target, source: source, status: status }
    }
}

#[derive(Debug)]
pub struct Conversions(Vec<Conversion>);

impl Conversions {
    pub fn from_sources(s: Sources) -> Conversions {
        let paths: Vec<_> = s.iter().map(|s| s.path.clone()).collect();

        use std::ffi::OsString;
        let (base_path_len, base_path_new): (usize, PathBuf) = {
            let mut base_path: Vec<OsString> = get_longest_prefix(&paths).into_iter().map(|x| x.to_os_string()).collect();
            let base_path_len = base_path.len();
            let mut folder_name = base_path.pop().unwrap().to_os_string();
            folder_name.push(" - Converted");
            base_path.push(folder_name);
            (base_path_len, base_path.into_iter().collect())
        };

        let conversions = (paths).into_iter().zip(s).zip(0..).map(
            |((path, source), id)| Conversion::new(id, reprefix(&path, &base_path_new, base_path_len), source)
        );
        Conversions(conversions.collect())
    }
}

impl Deref for Conversions {
    type Target = [Conversion];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Conversions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

// fn reprefix_paths(p: &[PathBuf], new_prefix: &OsStr) -> Vec<PathBuf> {
//     let longest_prefix: usize = get_longest_prefix(&p);
//     p.into_iter().map(|x| reprefix(x, new_prefix, longest_prefix)).collect()

// }
fn reprefix(p: &PathBuf, new_prefix: &PathBuf, count: usize) -> PathBuf {
    let unprefixed: PathBuf = p.into_iter().skip(count).collect();
    new_prefix.join(unprefixed)
}

fn get_longest_prefix<'a>(paths: &'a [PathBuf]) -> Vec<&'a OsStr> {
    let components: Vec<_> = paths.into_iter().map(|p| {
        let mut p: Vec<_> = p.into_iter().collect();
        p.pop();
        p
    }).collect();

    let mut iter = components.iter();
    let longest = match iter.next() {
        Some(s) => s.as_slice(),
        None => &[]
    };

    let longest = iter.fold(longest, |longest, new| common_prefix(longest, new));

    (longest).into_iter().map(|&x| x).collect()
}
