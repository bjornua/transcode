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
extern crate rustc_serialize;
extern crate regex;

mod main {
    use source;
    use conversion;
    use ffmpeg;
    use args::{self, Args};

    pub enum ErrorKind {
        ArgError(args::Error),
        SourceError(source::Error)
    }

    pub fn main() -> Result<(), ErrorKind> {
        use self::ErrorKind::{ArgError, SourceError};

        let args = match Args::from_env() {
            Ok(a) => a,
            Err(e) => { return Err(ArgError(e)) }
        };
        let sources = match source::get_many(args.input) {
            Ok(s) =>  s,
            Err(e) => { return Err(SourceError(e)) }
        };

        let conversions = conversion::from_sources(sources);

        print_conversion(&conversions);
        println!("");
        // println!("{:#?}", conversions);
        ffmpeg::ffmpeg(&conversions[0]);

        Ok(())
    }

    pub fn main_handle_errors() {
        use error;
        match main() {
            Ok(()) => (),
            Err(e) => { error::print_error(e); return }
        }
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
        print_table(vec!["ID", "Path", "Duration", "MPixel"], data);
    }
}

fn main() {
    main::main_handle_errors();
}
