pub mod args;
pub mod conversion;
pub mod error;
pub mod ffmpeg;
pub mod ffprobe;
pub mod path;
pub mod progress;
pub mod regexreader;
pub mod source;
pub mod table;
pub mod target;
pub mod time;
pub mod utils;
extern crate regex;
extern crate rustc_serialize;

mod main {
    use args::{self, Args};
    use conversion::{Conversions, Conversion};
    use error;
    use ffmpeg;
    use progress::Status;
    use source;
    use std::iter::once;
    use std::path::PathBuf;
    use table::Alignment::{Left, Right};
    use table::Cell::{Text, Empty, self};
    use table::print_table;
    use time::pretty_centiseconds;
    use utils::erase_up;

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
        if let Err(e) = main() {
            error::print_error(e)
        }
    }

    // pub fn new_main() {
    //     use path::{RecursivePathIterator, PathType};

    //     let count = RecursivePathIterator::new("/home/bjorn").take(1_000_000).fold(
    //         (0, 0), |(a, b), x| match x {
    //             PathType::Directory(_) => (a + 1, b    ),
    //             PathType::File(_) =>      (a    , b + 1)
    //     });
    //     println!("{:?}", count);
    // }

    pub fn main() -> Result<(), Error> {
        let args = try!(Args::from_env());
        println!("{:#?}", args);
        let sources = try!(source::get_many(args.input));
        let conversions = Conversions::from_sources(sources);

        try!(convert(conversions));

        Ok(())
    }

    fn print_conversion<'a>(conversions: &Conversions, status: &Status) -> usize {

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
            vec![
                Text(Left(c.target.path.to_string_lossy())),
                Text(Left((&c.status).into())),
                eta(&c.status),
                Text(Left((c.status.bar(20)).into()))
            ]
        }
        let data = conversions.into_iter().map(row).chain(once(vec![]));
        let data_sum = data.chain(once(vec![
            Text(Left("Total".into())),
            Text(Left(status.into())),
            eta(status),
            Text(Left(status.bar(20).into())),
        ]));
        print_table(Some(vec!["Path", "Status", "Eta", ""]), data_sum)
    }

    fn convert(mut conversions: Conversions) -> Result<(), (ffmpeg::Error)> {
        let global_mpixel: f64 = (&conversions).into_iter().map(|c| c.source.ffprobe.mpixel()).sum();
        let mut global_status = Status::new(global_mpixel);
        let mut global_progress = 0.;
        let mut lines = print_conversion(&conversions, &global_status);


        global_status.start();
        for n in 0..conversions.len() {
            // Okay, hope this scope thing is going to be better in the future :)
            let (local_mpixel, path): (f64, PathBuf) = {
                let ref mut c = conversions[n];
                c.status.start();
                (c.source.ffprobe.mpixel(), c.source.path.clone())
            };
            for time in try!(ffmpeg::FFmpegIterator::new(path.as_ref())) {
                {
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
