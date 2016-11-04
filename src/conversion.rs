use ffmpeg;
use progress::{Status, status_sum};
use source::{Sources, Source};
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::path::{PathBuf, Path};
use target;
use utils::erase_up;

#[derive(Debug, Clone)]
pub struct Conversion {
    pub id: u64,
    pub source: Source,
    pub target: target::Target,
    pub status: Status,
}

#[derive(Debug)]
pub enum Error {
    FFmpegError {
        conversion: Conversion,
        error: ffmpeg::Error,
    },
    TargetError(target::Error),
}


impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::TargetError(_) => "Target error",
            Error::FFmpegError { .. } => "FFmpeg error",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::TargetError(ref error) => Some(error),
            Error::FFmpegError { ref error, .. } => Some(error),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::TargetError(_) => write!(f, "{}", self.description()),
            Error::FFmpegError { ref conversion, .. } => {
                write!(f, "{}: {:?}", self.description(), conversion)
            }
        }

    }
}

impl Conversion {
    pub fn new(id: u64, target: target::Target, source: Source) -> Self {
        let status = Status::new(source.ffprobe.mpixel());

        Conversion {
            id: id,
            target: target,
            source: source,
            status: status,
        }
    }
}

#[derive(Debug)]
pub struct Conversions(Vec<Conversion>);

impl Conversions {
    pub fn from_sources(s: Sources,
                        target_dir: &str)
                        -> Result<(Conversions, Vec<PathBuf>), Error> {
        let target_dir = Path::new(&target_dir);
        let extension = OsStr::new("mkv");

        if s.len() == 0 {
            return Ok((Conversions(Vec::new()), Vec::new()));
        }

        let sources = s.into_iter()
            .map(|source| {
                target::Target::new(target_dir, &source.path.relative(), extension)
                    .map(|t| (t, source))
            })
            .map(|result| {
                match result {
                    Ok(s) => Ok(Ok(s)),
                    Err(target::Error::Exists { path }) => Ok(Err(path)),
                    Err(e) => Err(Error::TargetError(e)),
                }
            });

        let sources: Result<Vec<_>, Error> = sources.collect();

        let (good, skipped): (Vec<_>, Vec<_>) = try!(sources).into_iter().partition(|r| r.is_ok());
        let good = good.into_iter().filter_map(|s| s.ok());
        let skipped = skipped.into_iter().filter_map(|s| s.err());

        let conversions = good.zip(0..)
            .map(|((target, source), id)| Conversion::new(id, target, source));



        Ok((Conversions(conversions.collect()), skipped.collect()))
    }
    pub fn print_table(&self) -> usize {
        use table::print_table;
        use table::Cell::{self, Text, Empty, Integer};
        use table::Alignment::{Left, Right};
        use time::pretty_centiseconds;
        use strings::truncate_left;
        use std::iter::once;
        use std::borrow::Cow;

        fn seconds_to_cell<'a>(n: f64) -> Cell<'a> {
            Text(Right(pretty_centiseconds((n * 100.).round() as i64).into()))
        }
        fn eta<'a, 'b>(s: &'b Status) -> Cell<'a> {
            match *s {
                Status::Pending(_) => Empty,
                Status::Progress(ref p) => p.eta().map_or(Empty, seconds_to_cell),
                Status::Done(ref p) => seconds_to_cell(p.duration),
            }
        }

        fn row<'a>(c: &'a Conversion) -> Vec<Cell<'a>> {
            vec![
                Integer(Cow::Owned(c.id as i64)),
                Text(Left(truncate_left(c.target.path.to_string_lossy(), "...", 60))),
                Text(Left((&c.status).into())),
                eta(&c.status),
            ]
        }

        let conversions = self.into_iter()
            .filter(|c| match c.status {
                Status::Progress(_) => true,
                _ => false,
            })
            .map(row)
            .chain(once(vec![]));


        let global_status: Option<Status> = status_sum(self.into_iter()
            .map(|&Conversion { ref status, .. }| status));

        let sums = match global_status {
            Some(ref global_status) => {
                let count = self.into_iter().count();
                vec![vec![],
                     vec![Integer(Cow::Owned(count as i64)),
                          Text(Left("Total".into())),
                          Text(Left(global_status.into())),
                          eta(global_status),
                      ]]
            }
            None => vec![],
        };

        let data = conversions.chain(sums);

        print_table(Some(vec!["Num", "Path", "Status", "Eta", ""]), data)
    }

    pub fn convert(mut self, dry_run: bool) -> Result<(), Error> {
        let mut lines = 0;
        for n in 0..self.len() {
            // Okay, hope this scope thing is going to be better in the future :)
            let (local_mpixel, ffmpeg_con): (f64, Conversion) = {
                let ref mut c = self[n];
                (c.source.ffprobe.mpixel(), c.clone())
            };

            if !dry_run {
                match ffmpeg_con.target.mkdir_parent() {
                    Ok(()) => (),
                    Err(e) => return Err(Error::TargetError(e)),
                }
            }

            match ffmpeg_con.target.remove_path_tmp() {
                Err(e) => return Err(Error::TargetError(e)),
                Ok(true) | Ok(false) => (),
            }

            let ffmpegiter = match ffmpeg::FFmpegIterator::new(&ffmpeg_con, dry_run) {
                Ok(iter) => iter,
                Err(e) => {
                    return Err(Error::FFmpegError {
                        conversion: ffmpeg_con,
                        error: e,
                    })
                }
            };

            for time in ffmpegiter {
                {
                    let time = match time {
                        Ok(t) => t,
                        Err(e) => {
                            return Err(Error::FFmpegError {
                                conversion: ffmpeg_con,
                                error: e,
                            })
                        }
                    };
                    let ref mut c = self[n];
                    let local_progress = time / c.source.ffprobe.duration * local_mpixel;
                    c.status.update(local_progress);
                }
                erase_up(lines);
                lines = self.print_table();
            }
            {
                match ffmpeg_con.target.rename_path_tmp() {
                    Err(e) => return Err(Error::TargetError(e)),
                    Ok(()) => (),
                }
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
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Conversions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


// fn get_longest_prefix<'a>(paths: &'a [PathBuf]) -> Vec<&'a OsStr> {
//     let components: Vec<_> = paths.into_iter()
//         .map(|p| {
//             let mut p: Vec<_> = p.into_iter().collect();
//             p.pop();
//             p
//         })
//         .collect();

//     let mut iter = components.iter();
//     let longest = match iter.next() {
//         Some(s) => s.as_slice(),
//         None => &[],
//     };

//     let longest = iter.fold(longest, |longest, new| common_prefix(longest, new));

//     (longest).into_iter().map(|&x| x).collect()
// }
