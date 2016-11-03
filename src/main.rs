pub mod args;
pub mod conversion;
pub mod check_file;
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
pub mod constants;
extern crate regex;
extern crate rustc_serialize;
extern crate getopts;

use std::process::exit;

pub fn main() {
    let exit_code = match run() {
        Err(error::Error::NoSourcesError) => {
            error::print_error(&error::Error::NoSourcesError);
            0
        }
        Err(e) => {
            error::print_error(&e);
            1
        }
        Ok(()) => 0,
    };
    exit(exit_code)
}

pub fn run() -> Result<(), error::Error> {
    let args = try!(args::Args::from_env());

    if args.help {
        ::args::print_usage(&args.program_name);
        return Ok(());
    }

    let (sources, bads) = try!(source::Sources::from_paths(args.paths));

    if bads.len() > 0 {
        print_bads(bads.as_slice());
        println!("");
    }

    if sources.len() == 0 {
        return Err(error::Error::NoSourcesError);
    }

    let conversions = try!(conversion::Conversions::from_sources(sources));

    print_sources(&conversions);
    println!("");

    if utils::prompt_continue() {
        println!("");
        try!(conversions.convert(args.dry_run));
    }

    Ok(())
}

use std::path::PathBuf;
fn print_bads(bads: &[PathBuf]) {
    println!("Skipping non video/audio files:");
    for path in bads {
        println!("      {}", path.to_string_lossy());
    }
}

fn print_sources(sources: &conversion::Conversions) {
    println!("Converting: ");
    for con in sources.iter() {
        println!("{: >4}: {}", con.id, con.source.path.to_string_lossy());
    }
}
