use source::Source;
use target::Target;
use std::path::PathBuf;
use std::ffi::OsStr;
use progress::Status;


#[derive(Debug, Clone)]
pub struct Conversion {
    pub id: u64,
    pub source: Source,
    pub target: Target,
    pub status: Status
}

impl Conversion {
    pub fn new(id: u64, path: PathBuf, source: Source) -> Self {
        let status = Status::new(source.ffprobe.mpixel());
        let target = Target {path: path};
        Conversion { id: id, target: target, source: source, status: status }
    }
    pub fn get_progress(&self, time_processed: f64) -> f64 {
        time_processed / self.source.ffprobe.duration
    }

}


pub fn from_sources(s: Vec<Source>) -> Vec<Conversion> {
    let paths: Vec<_> = (&s).into_iter().map(|s| (s.path.clone())).collect();

    unprefix_paths(&paths).into_iter().zip(s).zip(0..)
        .map(|((path, source), id)| Conversion::new(id, path, source)
    ).collect()
}


fn unprefix_paths(p: &[PathBuf]) -> Vec<PathBuf> {
    fn unprefix(p: &PathBuf, count: usize) -> PathBuf {
        p.into_iter().skip(count).collect::<PathBuf>()
    }
    pub fn get_longest_prefix(s: &[PathBuf]) -> usize {
        fn pathbuf_to_osstr<'a>(p: &'a PathBuf) -> Vec<&'a OsStr> {
            p.into_iter().collect::<Vec<_>>()
        }
        fn common_prefix<'a>(a: &'a [&'a OsStr], b: &[&OsStr]) -> &'a [&'a OsStr] {
            let count = a.into_iter().zip(b.into_iter()).take_while(|&(ref a, ref b)| a == b).count();

            return &a[0..count]
        }
        fn get_prefix<'a>(s: &'a [&'a OsStr]) -> &'a [&'a OsStr] {
            match s.split_last() {
                Some((_, s)) => s,
                None => s
            }
        }

        let ss: Vec<Vec<&OsStr>> = s.into_iter().map(pathbuf_to_osstr).collect();

        let mut s = (&ss).into_iter().map(|ref x| get_prefix((x)));
        let longest = match s.next() {
            Some(s) => s,
            None => return 0
        };

        let longest = s.fold(longest, |longest, ref new| common_prefix(longest, new));

        longest.len()
    }

    let longest_prefix: usize = get_longest_prefix(&p);

    p.into_iter().map(|x| unprefix(x, longest_prefix)).collect()
}
