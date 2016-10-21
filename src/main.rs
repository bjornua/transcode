pub mod regexreader;
pub mod ffprobe;
pub mod time;
pub mod source;
pub mod target;
pub mod conversion;
pub mod table;
pub mod args;
pub mod error;
pub mod ffmpeg;
pub mod utils;

extern crate rustc_serialize;
extern crate regex;

mod main {
    use source;
    use conversion;
    use ffmpeg;
    use args::{self, Args};

    pub enum Error {
        ArgError(args::Error),
        SourceError(source::Error),
        FFmpegError(ffmpeg::Error)
    }
    impl From<ffmpeg::Error> for Error {
        fn from(err: ffmpeg::Error) -> Self {
            Error::FFmpegError(err)
        }
    }
    impl From<args::Error> for Error {
        fn from(err: args::Error) -> Self {
            Error::ArgError(err)
        }
    }
    impl From<source::Error> for Error {
        fn from(err: source::Error) -> Self {
            Error::SourceError(err)
        }
    }

    pub fn main_handle_errors() {
        use error;
        match main() {
            Ok(()) => (),
            Err(e) => { error::print_error(e); return }
        }
    }

    pub fn main() -> Result<(), Error> {
        let args = try!(Args::from_env());
        let sources = try!(source::get_many(args.input));
        let conversions = conversion::from_sources(sources);

        print_conversion(&conversions);
        println!("");

        try!(convert(&conversions));

        Ok(())
    }

    fn print_conversion<'a>(conversions: &[conversion::Conversion]) {
        use table::Alignment::{Left, Right};
        use table::Cell::{Integer, Text, Float, Empty, self};
        use table::print_table;
        use time::pretty_centiseconds;
        use std::borrow::Cow::{Owned,Borrowed};
        use std::iter::once;

        let (s_duration, s_fps) = conversions.into_iter().fold((0., 0.), |(a, b), c |
            (a + c.source.ffprobe.duration, b + c.source.ffprobe.mpixel())
        );

        fn seconds_to_cell<'a>(n: f64) -> Cell<'a> {
            Text(Right(Owned(pretty_centiseconds((n * 100.).round() as i64))))
        }

        let data = conversions.into_iter().map(|c| vec![
            Integer(Owned(c.id as i64)),
            Text(Left(Borrowed(c.target.path.to_str().unwrap()))),
            seconds_to_cell(c.source.ffprobe.duration),
            Float(Owned(c.source.ffprobe.mpixel()), 0),
        ]).chain(once(vec![])).chain(once(vec![
            Empty,
            Text(Left(Borrowed("Total"))),
            seconds_to_cell(s_duration),
            Float(Owned(s_fps), 0)
        ]));
        print_table(Some(vec!["ID", "Path", "Duration", "MPixel"]), data);
    }
    fn format_bar(ratio: f64, width: usize) -> String {
        use std::cmp::{max, min};
        use std::borrow::Cow::{Owned, Borrowed};
        use utils;

        let width = width - 2;
        let bars = (ratio * (width as f64)).floor() as usize;
        let bars = max(0, min(bars, width));
        [
            Borrowed("["),
            Owned(utils::repeat_str("#", bars)),
            Owned(utils::repeat_str(" ", width - bars)),
            Borrowed("]")
        ].concat()
    }
    use std::time::Instant;
    fn get_stats(start: Instant, processed: f64, target: f64) -> (f64, f64) {
        let ratio = processed / target;

        let eta = {
            let elapsed_time = ((start.elapsed() * 1_000_000_000).as_secs() as f64) / 1_000_000_000.;
            let remaining = target - processed;
            let speed = processed / elapsed_time;
            remaining / speed
        };
        (ratio, eta)
    }
    fn print_stats(path: Option<&str>, start: Instant, processed: f64, target: f64) -> () {
        use table::Alignment::{Left, Right};
        use table::Cell::{Text};
        use std::borrow::Cow::{Owned,Borrowed};
        use time::pretty_centiseconds;
        use table::print_table;


        let (ratio, eta) = get_stats(start, processed, target);

        let mut stats = Vec::new();

        if let Some(p) = path {
            stats.push(vec![Text(Left(Borrowed("Path"))), Text(Left(Borrowed(p)))]);
        }

        stats.push(vec![Text(Left(Borrowed("Progress"))), Text(Left(Owned(format_bar(ratio, 40))))]);
        stats.push(vec![Text(Left(Borrowed("% Done"))), Text(Right(Owned(format!("{:0.2}%", ratio*100.))))]);
        stats.push(vec![Text(Left(Borrowed("Eta"))), Text(Right(Owned(pretty_centiseconds((eta*100.) as i64))))]);


        print_table(None, stats.into_iter());

    }
    fn erase_up(lines: usize) {
        for _ in 0..lines {
            print!("\x1B[2K\x1B[A");
        }
        print!("\r");
    }

    fn convert(conversion: &[conversion::Conversion]) -> Result<(), (ffmpeg::Error)> {
        use std::time::Instant;

        let global_mpixel: f64 = conversion.into_iter().map(|c| c.source.ffprobe.mpixel()).sum();
        let global_begin = Instant::now();
        let mut global_progress = 0.;

        print!("\n\n\n\n\n\n\n\n\n\n");
        for c in conversion {
            let local_mpixel = c.source.ffprobe.mpixel();
            let local_begin = Instant::now();
            for time in try!(ffmpeg::FFmpegIterator::new(c)) {
                erase_up(10);

                let local_progress = (time / c.source.ffprobe.duration * local_mpixel).max(0.).min(local_mpixel);
                println!("This file:");
                print_stats(Some(c.source.path.to_str().unwrap()), local_begin, local_progress, local_mpixel);
                println!("\nAll files:");
                print_stats(None, global_begin, global_progress + local_progress, global_mpixel);



                // print!("Eta:        {eta}\nProcessing: {id} {path}\nThis file:  {loc_pct:06.2}% {loc_bar}\nTotal:      {glob_pct:06.2}% {glob_bar}",
                //     loc_id=c.id,
                //     loc_path=c.source.path.to_str().unwrap(),
                //     loc_pct=local_ratio * 100.,
                //     loc_bar=print_bar(local_ratio, 60),
                //     glob_eta=pretty_centiseconds((global_eta * 100.) as i64),
                //     local_eta=pretty_centiseconds((local_eta * 100.) as i64),
                //     glob_pct=global_ratio * 100.,
                //     glob_bar=print_bar(global_ratio, 60)
                // );
                // let _ = stdout().flush();
            }
            global_progress = global_progress + local_mpixel;
            // println!("{}", global_progress);
        };
        print!("\n");
        Ok(())
    }
}



fn main() {
    main::main_handle_errors();
}
