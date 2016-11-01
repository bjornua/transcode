use std::fmt;
use std::error::Error as StdError;
use getopts::{self, Options};

#[derive(Debug)]
pub enum Error {
    MissingProgramName,
    MissingInputs { program_name: String },
    GetOptsFail {
        program_name: String,
        error: getopts::Fail,
    },
}
use self::Error::*;

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            MissingProgramName => "Missing program name (argv[0])",
            MissingInputs { .. } => "No inputs specified",
            GetOptsFail { .. } => "Argument error",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            MissingProgramName => None,
            MissingInputs { .. } => None,
            GetOptsFail { ref error, .. } => Some(error),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


pub fn opts() -> Options {
    let mut opts = Options::new();
    opts.optflag("d", "dry-run", "No paths are created or updated");
    opts.optflag("h", "help", "Display this help and exit");
    opts
}

pub fn print_usage(program_name: &str) {
    let brief = format!("Usage: {} [OPTION]... [PATH]...", program_name);
    println!("{}", opts().usage(&brief));
}



#[derive(Debug)]
pub struct Args {
    pub program_name: String,
    pub paths: Vec<String>,
    pub dry_run: bool,
    pub help: bool,
}

impl Args {
    pub fn from_iter<T: IntoIterator<Item = String>>(args: T) -> Result<Args, Error> {
        let mut args = args.into_iter();

        let program_name = match args.next() {
            Some(s) => s.clone(),
            None => return Err(MissingProgramName),
        };


        let args = match opts().parse(args) {
            Ok(a) => a,
            Err(e) => {
                return Err(Error::GetOptsFail {
                    program_name: program_name,
                    error: e,
                })
            }
        };
        let dry_run = args.opt_present("dry-run");
        let help = args.opt_present("help");
        let paths = match args.free.len() {
            0 => vec![".".to_string()],
            _ => args.free,
        };


        Ok(Args {
            program_name: program_name,
            dry_run: dry_run,
            help: help,
            paths: paths,
        })
    }
    pub fn from_env() -> Result<Args, Error> {
        use std::env;
        Args::from_iter(env::args())
    }
    pub fn usage(program_name: String) -> String {
        return format!("{} path0 path1 path2", program_name);
    }
}
