use std::cmp::{max, min};
use utils;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Pending {
    target: f64,
}

#[derive(Debug, Clone)]
pub struct Progress {
    begin: Instant,
    target: f64,
    processed: f64,
}

#[derive(Debug, Clone)]
pub struct Done {
    pub begin: Instant,
    pub duration: f64,
    target: f64,
}

#[derive(Debug, Clone)]
pub struct Fail {
    pub begin: Instant,
    pub duration: f64,
    target: f64,
}

#[derive(Debug, Clone)]
pub enum Status {
    Pending(Pending),
    Progress(Progress),
    Done(Done),
    Fail(Fail),
}

// impl<A,B,C> Merge<B> for A {
//     fn merge(&self, other: &B) -> A {
//         other.merge(self);
//     }
// }

// impl Target for Progress {
//     fn target(&self) -> f64 {
//         self.target
//     }
// }

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
            return Some(0.);
        }
        let remaining = self.target - self.processed;

        let speed = match (self.processed, self.elapsed_ns()) {
            (0., _) | (_, 0) => return None,
            (p, e) => p / ((e as f64) / 1_000_000_000.),
        };
        Some(remaining / speed)
    }
    pub fn update(&mut self, processed: f64) {
        self.processed = processed;
    }
    pub fn end(&self) -> Done {
        Done {
            begin: self.begin,
            duration: (self.elapsed_ns() as f64) / 1_000_000_000.,
            target: self.target,
        }
    }
    pub fn fail(&self) -> Fail {
        Fail {
            begin: self.begin,
            duration: (self.elapsed_ns() as f64) / 1_000_000_000.,
            target: self.target,
        }
    }
}

impl Pending {
    pub fn start(&self) -> Progress {
        Progress {
            begin: Instant::now(),
            processed: 0.,
            target: self.target,
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
impl From<Fail> for Status {
    fn from(other: Fail) -> Self {
        Status::Fail(other)
    }
}


impl Status {
    pub fn new(target: f64) -> Self {
        Status::Pending(Pending { target: target })
    }
    pub fn start(&mut self) {
        *self = if let Status::Pending(ref s) = *self {
            s.start().into()
        } else {
            return;
        }
    }
    pub fn update(&mut self, progress: f64) {
        self.start();
        if let Status::Progress(ref mut s) = *self {
            s.update(progress)
        }
    }
    pub fn fail(&mut self) {
        *self = match *self {
            Status::Progress(ref s) => s.fail().into(),
            _ => return,
        }
    }
    pub fn get_processed(&self) -> f64 {
        match *self {
            Status::Progress(Progress { processed, .. }) => processed,
            Status::Done(Done { target, .. }) |
            Status::Fail(Fail { target, .. }) => target,
            Status::Pending(_) => 0.,
        }
    }
    pub fn get_target(&self) -> f64 {
        match *self {
            Status::Progress(Progress { target, .. }) |
            Status::Done(Done { target, .. }) |
            Status::Fail(Fail { target, .. }) |
            Status::Pending(Pending { target, .. }) => target,
        }
    }
    pub fn merge_begin(&self, &other: &Instant) -> Instant {
        match *self {
            Status::Pending(Pending { .. }) => other,
            Status::Progress(Progress { begin, .. }) |
            Status::Done(Done { begin, .. }) |
            Status::Fail(Fail { begin, .. }) => min(begin, other),

        }
    }
    pub fn end(&mut self) {
        *self = if let &mut Status::Progress(ref s) = self {
            s.end().into()
        } else {
            return;
        }
    }
    pub fn merge(&self, other: &Self) -> Self {
        let target = other.get_target() + self.get_target();
        let processed = other.get_processed() + self.get_processed();

        match *self {
            Status::Progress(Progress { begin, .. }) => {
                Progress {
                        begin: other.merge_begin(&begin),
                        processed: processed,
                        target: target,
                    }
                    .into()
            }
            Status::Pending(_) => {
                match *other {
                    Status::Pending(_) => Pending { target: target }.into(),
                    Status::Progress(_) => other.merge(self),
                    Status::Done(Done { ref begin, .. }) |
                    Status::Fail(Fail { ref begin, .. }) => {
                        Progress {
                                begin: *begin,
                                processed: processed,
                                target: target,
                            }
                            .into()
                    }
                }
            }
            Status::Done(Done { begin, duration, .. }) => {
                match *other {
                    Status::Done(ref s) => {
                        Done {
                                begin: min(begin, s.begin),
                                target: target,
                                duration: duration + s.duration,
                            }
                            .into()
                    }
                    Status::Pending(_) |
                    Status::Progress(_) => other.merge(self),
                    Status::Fail(Fail { begin, duration: duration_fail, .. }) => {
                        Fail {
                                duration: duration + duration_fail,
                                begin: begin,
                                target: target,
                            }
                            .into()
                    }
                }
            }
            Status::Fail(Fail { begin, duration, .. }) => {
                match *other {
                    Status::Fail(ref s) => {
                        Fail {
                                begin: min(begin, s.begin),
                                target: target,
                                duration: duration + s.duration,
                            }
                            .into()
                    }
                    Status::Pending(_) |
                    Status::Progress(_) |
                    Status::Done(_) => other.merge(self),
                }
            }
        }
    }
    pub fn bar(&self, width: usize) -> String {

        let width = width - 0;
        let bars = match *self {
            Status::Pending(_) => 0,
            Status::Done(_) | Status::Fail(_) => width,
            Status::Progress(ref s) => (s.ratio() * (width as f64)).floor() as usize,
        };

        let bars = max(0, min(bars, width));
        [Borrowed("["),
         Owned(utils::repeat_str("#", bars)),
         Owned(utils::repeat_str(" ", width - bars)),
         Borrowed("]")]
            .concat()
    }
}

pub fn status_sum<'a, T: IntoIterator<Item = &'a Status>>(statuses: T) -> Option<Status> {
    let mut statuses = statuses.into_iter();

    let mut global_status = match statuses.next() {
        Some(s) => s.clone(),
        None => return None,
    };

    for status in statuses {
        global_status = global_status.merge(status);
    }
    Some(global_status)
}


impl<'a> From<&'a Status> for Cow<'static, str> {
    fn from(s: &'a Status) -> Self {
        match *s {
            Status::Done(_) => Borrowed("Done"),
            Status::Fail(_) => Borrowed("Failed"),
            Status::Pending(_) => Borrowed("       "),
            Status::Progress(ref s) => Owned(format!("{:6.2}%", s.percentage())),
        }
    }
}
// impl<'a> From<Status> for Cow<'a, str> {
//     fn from(s: Status) -> Self {
//         match s {
//             Status::Pending(_) => Borrowed("       "),
//             Status::Done(_) => Borrowed("Done"),
//             Status::Progress(s) => Owned(format!("{:6.2}%", s.percentage())),
//         }
//     }
// }
