use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Pending {
    target: f64
}

#[derive(Debug, Clone)]
pub struct Progress {
    begin: Instant,
    target: f64,
    processed: f64
}

#[derive(Debug, Clone)]
pub struct Done {
    pub begin: Instant,
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
            begin: self.begin,
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
    pub fn get_progress(&self) -> f64 {
        match *self {
            Status::Progress(Progress { processed, ..} ) => processed,
            Status::Done(Done { target, ..} ) => target,
            Status::Pending(_) => 0.,
        }
    }
    pub fn get_target(&self) -> f64 {
        match *self {
            Status::Progress(Progress { target, ..} ) => target,
            Status::Done(Done { target, ..} ) => target,
            Status::Pending(Pending {target, ..}) => target,
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

pub fn status_sum<'a, T: IntoIterator<Item=&'a Status>>(statuses: T) -> Option<Status> {
    use std::cmp::min;
    let mut statuses = statuses.into_iter();

    let mut global_status = match statuses.next() {
        Some(s) => s.clone(),
        None => return None
    };

    for status in statuses {
        global_status = match (global_status, status) {
            (Status::Pending(ref g), &Status::Pending(ref s)) => {
                Pending { target: s.target +  g.target}.into()
            },
            (Status::Pending(ref g), &Status::Progress(ref s)) => {
                Progress { target: s.target + g.target, processed: s.processed, begin: s.begin}.into()
            },
            (Status::Pending(ref g), &Status::Done(ref s)) => {
                Progress { target: s.target + g.target, processed: s.target, begin: s.begin}.into()
            },
            (Status::Progress(ref g), &Status::Pending(ref s)) => {
                Progress { target: s.target + g.target, processed: g.processed, begin: g.begin}.into()
            },
            (Status::Progress(ref g), &Status::Progress(ref s)) => {
                Progress { target: s.target + g.target, processed: g.processed + s.processed, begin: min(s.begin, g.begin)}.into()
            },
            (Status::Progress(ref g), &Status::Done(ref s)) => {
                Progress { target: s.target + g.target, processed: g.processed + s.target, begin: min(s.begin, g.begin)}.into()
            },
            (Status::Done(ref g), &Status::Pending(ref s)) => {
                Progress { target: s.target + g.target, processed: g.target, begin: g.begin}.into()
            },
            (Status::Done(ref g), &Status::Progress(ref s)) => {
                Progress { target: s.target + g.target, processed: g.target, begin: min(s.begin, g.begin)}.into()
            },
            (Status::Done(ref g), &Status::Done(ref s)) => {
                Done { target: s.target + g.target, duration: g.duration + s.duration, begin: min(s.begin, g.begin)}.into()
            },
        }

    }
    Some(global_status)
}


impl<'a> From<&'a Status> for Cow<'a, str> {
    fn from(s: &'a Status) -> Self {
        match *s {
            Status::Pending(_) => Borrowed("       "),
            Status::Done(_) => Borrowed("Done"),
            Status::Progress(ref s) => Owned(format!("{:6.2}%", s.percentage()))
        }
    }
}
impl<'a> From<Status> for Cow<'a, str> {
    fn from(s: Status) -> Self {
        match s {
            Status::Pending(_) => Borrowed("       "),
            Status::Done(_) => Borrowed("Done"),
            Status::Progress(s) => Owned(format!("{:6.2}%", s.percentage()))
        }
    }
}

// impl Into<String> for Status {
//     fn into(self) -> String {

//     }
// }
