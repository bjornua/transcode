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
    use conversion::{Conversions};
    use error;
    use ffmpeg;
    use path;
    use source::{Source, Sources, self};
    use utils::{prompt};

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
        let cont = prompt(
            "Do you want to continue [y/n]?",
            |x| x == "y" || x == "n"
        ).map_or(false, |x| x == "y");

        if cont {
            println!("");
            let conversions = Conversions::from_sources(sources);
            try!(conversions.convert());
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
}


fn main() {
    main::main_handle_errors();
}
