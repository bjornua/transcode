use ffmpeg;
use progress::{Status, status_sum};
use source::{Sources, Source};
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use target::Target;
use utils::common_prefix;
use utils::{erase_up};

#[derive(Debug, Clone)]
pub struct Conversion {
    id: u64,
    pub source: Source,
    pub target: Target,
    pub status: Status
}

#[derive(Debug)]
pub enum Error {
    TargetDirExists { target_dir: PathBuf }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::TargetDirExists { .. } => "Target directory exists"
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::TargetDirExists { .. } => None
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TargetDirExists {ref target_dir} => write!(f, "{}: {}", self.description(), target_dir.to_string_lossy())
        }

    }
}



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
    pub fn from_sources(s: Sources) -> Result<Conversions, Error> {
        if s.len() == 0 {
            return Ok(Conversions(Vec::new()));
        }

        let paths: Vec<_> = s.iter().map(|s| s.path.clone()).collect();

        use std::ffi::OsString;
        let (base_path_len, target_dir): (usize, PathBuf) = {
            let mut base_path: Vec<OsString> = get_longest_prefix(&paths).into_iter().map(|x| x.to_os_string()).collect();
            let base_path_len = base_path.len();
            let mut folder_name = base_path.pop().unwrap_or_else(|| OsString::new());
            folder_name.push(" - Converted");
            base_path.push(folder_name);
            (base_path_len, base_path.into_iter().collect())
        };

        if target_dir.exists() {
            return Err(Error::TargetDirExists {target_dir: target_dir})
        }

        let target_paths = paths.into_iter().map(
            |path| convert_path(&path, &target_dir, base_path_len, "mkv".as_ref())
        );

        let conversions: Vec<_> = (target_paths).into_iter().zip(s).zip(0..).map(
            |((target_path, source), id)| Conversion::new(
                id,
                target_path,
                source
            )
        ).collect();

        Ok(Conversions(conversions))
    }
    pub fn print_table(&self) -> usize {
        use table::print_table;
        use table::Cell::{self, Text, Empty};
        use table::Alignment::{Left, Right};
        use time::pretty_centiseconds;
        use std::borrow::Cow;
        use strings::truncate_left;
        use std::iter::once;
        fn seconds_to_cell<'a>(n: f64) -> Cell<'a> {
            Text(Right(pretty_centiseconds((n * 100.).round() as i64).into()))
        }
        fn eta<'a>(s: &Status) -> Cell<'a> {
            match *s {
                Status::Pending(_) => Empty,
                Status::Progress(ref p) => { p.eta().map_or(Empty, seconds_to_cell) },
                Status::Done(ref p) => seconds_to_cell(p.duration)
            }
        }

        fn row<'a>(c: &'a Conversion) -> Vec<Cell<'a>> {
            let paths: Cow<'a, str> = match ::path::find_relative_cwd(c.target.path.as_path()) {
                Ok(p) => Cow::Owned(p.to_string_lossy().into_owned()),
                Err(_) => { c.target.path.to_string_lossy() }
            };

            vec![
                Text(Left(truncate_left(paths, "...", 60))),
                Text(Left((&c.status).into())),
                eta(&c.status),
            ]
        }

        let conversions = self.into_iter().map(row).chain(once(vec![]));

        let global_status: Option<Status> = status_sum(self.into_iter().map(|&Conversion { ref status, ..}| status));

        let sums = match global_status {
            Some(ref global_status) => vec![vec![], vec![
                Text(Left("Total".into())),
                Text(Left(global_status.into())),
                eta(global_status)
            ]],
            None => vec![]
        };

        let data = conversions.chain(sums);

        print_table(Some(vec!["Path", "Status", "Eta", ""]), data)
    }

    pub fn convert(mut self) -> Result<(), (ffmpeg::Error)> {
        let mut lines = 0;
        for n in 0..self.len() {
            // Okay, hope this scope thing is going to be better in the future :)
            let (local_mpixel, ffmpeg_con): (f64, Conversion) = {
                let ref mut c = self[n];
                (c.source.ffprobe.mpixel(), c.clone())
            };
            for time in try!(ffmpeg::FFmpegIterator::new(ffmpeg_con)) {
                {
                    let time = try!(time);
                    let ref mut c = self[n];
                    let local_progress = time / c.source.ffprobe.duration * local_mpixel;
                    c.status.update(local_progress);
                }
                erase_up(lines);
                lines = self.print_table();
            }
            {
                let ref mut c = self[n];
                c.status.end();
            };
            erase_up(lines);
            lines = self.print_table();
        }
        erase_up(lines);
        self.print_table();
        print!("\n");
        Ok(())
    }
}

impl Deref for Conversions {
    type Target = [Conversion];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Conversions {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

fn convert_path(p: &PathBuf, new_prefix: &PathBuf, count: usize, extension: &OsStr) -> PathBuf {
    let mut unprefixed: PathBuf = p.into_iter().skip(count).collect();
    unprefixed.set_extension(extension);
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
