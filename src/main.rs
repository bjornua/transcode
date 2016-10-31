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
    use source::{Source, Sources, self};
    use utils::{erase_up, prompt};

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
        let (sources, bads) = Sources::from_paths(args.input);


        if bads.len() > 0 {
            print_bads(bads.as_slice());
            println!("");
        }

        print_sources(&sources);
        println!("");
        let cont = prompt("Do you want to continue [y/n]?", |x| x == "y" || x == "n").map_or(false, |x| x == "y");
        println!("");


        if cont {
            let conversions = Conversions::from_sources(sources);
            try!(convert(conversions));
        }

        Ok(())
    }

    fn print_bads(bads: &[source::Error]) {
        println!("Skipping:");
        for error in bads {
            let path = match *error {
                source::Error::FFProbeError { ref path,.. }
                | source::Error::PathError {ref path,..} => {
                    path::find_relative_cwd(path).ok()
                }
            };
            println!("    {}", path.as_ref().map(|p| p.to_string_lossy()).unwrap_or("".into()));
        }
    }
    fn print_sources(sources: &Sources) {
        println!("Converting: ");
        for &Source { ref path, ..} in sources.iter() {
            println!("    {}", path.to_string_lossy());
        }
    }

    fn convert(mut conversions: Conversions) -> Result<(), (ffmpeg::Error)> {
        let mut lines = 0;
        for n in 0..conversions.len() {

            // Okay, hope this scope thing is going to be better in the future :)
            let (local_mpixel, ffmpeg_con): (f64, conversion::Conversion) = {
                let ref mut c = conversions[n];
                (c.source.ffprobe.mpixel(), c.clone())
            };
            for time in try!(ffmpeg::FFmpegIterator::new(ffmpeg_con)) {
                {
                    let time = try!(time);
                    let ref mut c = conversions[n];
                    let local_progress = time / c.source.ffprobe.duration * local_mpixel;
                    c.status.update(local_progress);
                }
                erase_up(lines);
                lines = conversions.print_table();
            }
            {
                let ref mut c = conversions[n];
                c.status.end();
            };
            erase_up(lines);
            lines = conversions.print_table();
        }
        erase_up(lines);
        conversions.print_table();

        print!("\n");
        Ok(())
    }
}


fn main() {
    main::main_handle_errors();
}
