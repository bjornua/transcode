use progress::Status;
use source::{Sources, Source};
use std::ffi::OsStr;
use std::iter::once;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use target::Target;


#[derive(Debug, Clone)]
pub struct Conversion {
    id: u64,
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
}

pub struct Conversions(Vec<Conversion>);

impl Conversions {
    pub fn from_sources(s: Sources) -> Conversions {
        let paths: Vec<_> = s.iter().map(|s| (s.path.clone())).collect();

        Conversions(
            reprefix_paths(&paths, OsStr::new("")).into_iter().zip(s).zip(0..).map(
            |((path, source), id)| Conversion::new(id, path, source)
        ).collect())
    }
}
impl Deref for Conversions {
    type Target = [Conversion];
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl DerefMut for Conversions {
    // type Target = [Conversion];
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}


fn reprefix_paths(p: &[PathBuf], new_prefix: &OsStr) -> Vec<PathBuf> {
    let longest_prefix: usize = get_longest_prefix(&p);
    p.into_iter().map(|x| reprefix(x, new_prefix, longest_prefix)).collect()

}
fn reprefix(p: &PathBuf, new_prefix: &OsStr, count: usize) -> PathBuf {
    let unprefixed = p.into_iter().skip(count);
    once(new_prefix).chain(unprefixed).collect::<PathBuf>()
}

fn get_longest_prefix<'a>(paths: &'a [PathBuf]) -> usize {
    let components: Vec<_> = paths.into_iter().map(|p| {
        let mut p: Vec<_> = p.into_iter().collect();
        p.pop();
        p
    }).collect();

    let mut iter = components.iter();

    let longest = match iter.next() {
        Some(s) => s.as_slice(),
        None => return 0
    };

    let longest = iter.fold(longest, |longest, new| common_prefix(longest, new));

    longest.len()
}

fn common_prefix<'a, T: PartialEq>(a: &'a[T], b: &[T]) -> &'a[T] {
    let common = a.into_iter().zip(b.into_iter()).take_while(
        |&(a, b)| a == b
    );

    return &a[0..common.count()]
}
