pub mod args;
pub mod conversion;
pub mod error;
pub mod ffmpeg;
pub mod ffprobe;
pub mod path;
pub mod progress;
pub mod regexreader;
pub mod source;
pub mod strings;
pub mod table;
pub mod target;
pub mod time;
pub mod utils;
extern crate regex;
extern crate rustc_serialize;

mod main {
    use args::{self, Args};
    use conversion::{Conversions, self};
    use error;
    use ffmpeg;
    use path;
    use std::borrow::Cow;
    use progress::Status;
    use source::{Sources, self};
    use std::iter::once;
    use strings::{truncate_left};
    use table::Alignment::{Left, Right};
    use table::Cell::{Text, Empty, self};
    use table::print_table;
    use time::pretty_centiseconds;
    use utils::{erase_up};

    pub enum Error {
        ArgError(args::Error),
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

    pub fn main_handle_errors() {
        if let Err(e) = main() {
            error::print_error(e)
        }
    }

    pub fn main() -> Result<(), Error> {
        let args = try!(Args::from_env());
        // println!("{:#?}", args);
        let (sources, bads) = Sources::from_paths(args.input);

        // println!("{:#?}", sources);
        // return Ok(());
        let conversions = Conversions::from_sources(sources);

        if bads.len() > 0 {
            print_bads(bads.as_slice());
        }

        try!(convert(conversions));

        Ok(())
    }

    fn print_bads(bads: &[source::Error]) {
        for error in bads {
            let path = match *error {
                source::Error::FFProbeError { ref path,.. }
                | source::Error::PathError {ref path,..} => {
                    path::find_relative_cwd(path).ok()
                }
            };
            print!("Skipping: {}\n", path.as_ref().map(|p| p.to_string_lossy()).unwrap_or("".into()));
        }
        println!("");
    }

    fn print_conversion(conversions: &[conversion::Conversion], status: &Status) -> usize {
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

        fn row<'a>(c: &'a conversion::Conversion) -> Vec<Cell<'a>> {
            let paths: Cow<'a, str> = match ::path::find_relative_cwd(c.target.path.as_path()) {
                Ok(p) => Cow::Owned(p.to_string_lossy().into_owned()),
                Err(_) => { c.target.path.to_string_lossy() }
            };

            vec![
                Text(Left(truncate_left(paths, "←", 60))),
                Text(Left((&c.status).into())),
                eta(&c.status),
                // Text(Left((c.status.bar(20)).into()))
            ]
        }
        let conversions = conversions.into_iter().map(row).chain(once(vec![]));
        let sums = once(vec![
            Text(Left("Total".into())),
            Text(Left(status.into())),
            eta(status),
            // Text(Left(status.bar(20).into())),
        ]);

        let data = conversions.chain(once(vec![])).chain(sums);

        print_table(Some(vec!["Path", "Status", "Eta", ""]), data)
    }

    fn convert(mut conversions: Conversions) -> Result<(), (ffmpeg::Error)> {
        let global_mpixel: f64 = (&conversions).into_iter().map(
            |c| c.source.ffprobe.mpixel()
        ).sum();

        let mut global_status = Status::new(global_mpixel);
        let mut global_progress = 0.;
        let mut lines = print_conversion(&conversions, &global_status);

        global_status.start();
        for n in 0..conversions.len() {

            // Okay, hope this scope thing is going to be better in the future :)
            let (local_mpixel, ffmpeg_con): (f64, conversion::Conversion) = {
                let ref mut c = conversions[n];
                c.status.start();
                (c.source.ffprobe.mpixel(), c.clone())
            };
            for time in try!(ffmpeg::FFmpegIterator::new(ffmpeg_con)) {
                {
                    let time = try!(time);
                    let ref mut c = conversions[n];
                    let local_progress = time / c.source.ffprobe.duration * local_mpixel;
                    c.status.update(local_progress);
                    global_status.update(global_progress + local_progress);
                }
                erase_up(lines);
                lines = print_conversion(&conversions, &global_status);
            }
            global_progress += local_mpixel;
            {
                let ref mut c = conversions[n];
                c.status.end();
            };
            erase_up(lines);
            lines = print_conversion(&conversions, &global_status);
        }
        global_status.end();

        erase_up(lines);
        print_conversion(&conversions, &global_status);

        print!("\n");
        Ok(())
    }
}


fn main() {
    main::main_handle_errors();
}
