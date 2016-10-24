use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Progress {
    target: f64,
    begin: Instant,
    processed: f64
}

#[derive(Debug, Clone)]
pub struct Pending {
    target: f64
}

#[derive(Debug, Clone)]
pub struct Done {
    pub duration: f64,
    target: f64
}

#[derive(Debug, Clone)]
pub enum Status {
    Pending(Pending),
    Progress(Progress),
    Done(Done)
}

impl Progress {
    pub fn percentage(&self) -> f64 {
        (self.ratio() * 100.).max(0.).min(100.)
    }
    pub fn ratio(&self) -> f64 {
        (self.processed / self.target).max(0.).min(1.)
    }
    pub fn elapsed_ns(&self) -> u64 {
        (self.begin.elapsed() * 1_000_000_000).as_secs()
    }
    pub fn eta(&self) -> Option<f64> {
        if self.processed >= self.target {
            return Some(0.)
        }
        let remaining = self.target - self.processed;

        let speed = match (self.processed, self.elapsed_ns()) {
            (0., _) | (_, 0) => return None,
            (p, e) => p / ((e as f64) / 1_000_000_000.)
        };
        Some(remaining / speed)
    }
    pub fn update(&mut self, processed: f64) {
        self.processed = processed;
    }
    pub fn end(&self) -> Done {
        Done {
            duration: (self.elapsed_ns() as f64) / 1_000_000_000.,
            target: self.target
        }
    }

}

impl Pending {
    pub fn start(&self) -> Progress {
        Progress {
            begin: Instant::now(),
            processed: 0.,
            target: self.target
        }
    }
}

impl From<Pending> for Status {
    fn from(other: Pending) -> Self {
        Status::Pending(other)
    }
}
impl From<Progress> for Status {
    fn from(other: Progress) -> Self {
        Status::Progress(other)
    }
}
impl From<Done> for Status {
    fn from(other: Done) -> Self {
        Status::Done(other)
    }
}

impl Status {
    pub fn new(target: f64) -> Self {
        Status::Pending(Pending {target: target})
    }
    pub fn start(&mut self) {
        *self = if let &mut Status::Pending(ref s) = self {
            s.start().into()
        } else {
            return
        }
    }
    pub fn update(&mut self, progress: f64) {
        if let &mut Status::Progress(ref mut s) = self {
            s.update(progress)
        }
    }
    pub fn end(&mut self) {
        *self = if let &mut Status::Progress(ref s) = self {
            s.end().into()
        } else {
            return
        }
    }
    pub fn bar(&self, width: usize) -> String {
        use std::cmp::{max, min};
        use std::borrow::Cow::{Owned, Borrowed};
        use utils;

        let width = width - 0;
        let bars = match *self {
            Status::Pending(_) => 0,
            Status::Done(_) => width,
            Status::Progress(ref s) => (s.ratio() * (width as f64)).floor() as usize
        };

        let bars = max(0, min(bars, width));
        [
            Borrowed("["),
            Owned(utils::repeat_str("#", bars)),
            Owned(utils::repeat_str(" ", width - bars)),
            Borrowed("]")
        ].concat()
    }
}
use std::borrow::Cow::{self, Borrowed, Owned};

impl<'a> From<&'a Status> for Cow<'a, str> {
    fn from(s: &'a Status) -> Self {
        match *s {
            Status::Pending(_) => Borrowed("       "),
            Status::Done(_) => Borrowed("Done"),
            Status::Progress(ref s) => Owned(format!("{:6.2}%", s.percentage()))
        }
    }
}

// impl Into<String> for Status {
//     fn into(self) -> String {

//     }
// }
