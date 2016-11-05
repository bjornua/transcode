pub mod args;
pub mod check_file;
pub mod constants;
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
extern crate getopts;
extern crate regex;
extern crate rustc_serialize;

use std::path::{Path, PathBuf};
use std::process::exit;

pub fn main() {
    let exit_code = match run() {
        Err(error::Error::NoSourcesError) => {
            error::print_error(&error::Error::NoSourcesError);
            0
        }
        Err(error::Error::ArgError(args::Error::Help { program_name })) => {
            ::args::print_usage(&program_name);
            0
        }
        Err(e) => {
            error::print_error(&e);
            1
        }
        Ok(false) => 1,
        Ok(true) => 0,
    };
    exit(exit_code)
}

pub fn run() -> Result<bool, error::Error> {
    let args = try!(args::Args::from_env());

    let (sources, bads) = try!(source::Sources::from_paths(args.paths, &args.source_dir));
    let (conversions, skipped) = try!(conversion::Conversions::from_sources(sources,
                                                                            &args.target_dir));
    print_bads(&bads);

    print_skipped(skipped.as_slice());

    if conversions.len() == 0 {
        return Err(error::Error::NoSourcesError);
    }

    print_conversions(&conversions, &args.target_dir);

    let mut fail = false;
    if utils::prompt_continue() {
        println!("");
        conversions.convert(args.dry_run, |err| {
            fail = true;
            error::print_error(&err.into())
        })
    }
    Ok(fail)
}

fn print_bads(paths: &[source::BasedPath]) {
    if paths.len() == 0 {
        return;
    }
    println!("Skipping non video/audio files:");
    for path in paths {
        println!("      {}", path.relative().to_string_lossy());
    }
    println!("");
}

fn print_skipped(paths: &[PathBuf]) {
    if paths.len() == 0 {
        return;
    }
    println!("Skipping existing targets:");
    for path in paths.into_iter() {
        println!("      {}", path.to_string_lossy());
    }
    println!("");
}

fn print_conversions(conversions: &conversion::Conversions, dir: &str) {
    if conversions.len() == 0 {
        return;
    }

    if let Ok(dir) = path::normalize(Path::new(dir)) {
        println!("Converting to {:?}: ", dir);
    } else {
        println!("Converting: ");
    }

    for con in conversions.iter() {
        println!("{: >4}: {}",
                 con.id,
                 con.source.path.relative().to_string_lossy());
    }
    println!("");
}
