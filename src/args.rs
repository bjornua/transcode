use std::fmt;
use std::error::Error as StdError;
use getopts::{self, Options};

#[derive(Debug)]
pub enum Error {
    MissingProgramName,
    MissingTargetDir { program_name: String },
    MissingSourceDir { program_name: String },
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
            MissingTargetDir { .. } => "No TARGET_DIRECTORY specified",
            MissingSourceDir { .. } => "No SOURCE_DIRECTORY specified",
            GetOptsFail { .. } => "Argument error",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            MissingProgramName => None,
            MissingTargetDir { .. } => None,
            MissingSourceDir { .. } => None,
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
    let brief = format!("Usage: {} [OPTION]... TARGET_DIRECTORY SOURCE_DIRECTORY [SOURCE_FILE]...",
                        program_name);
    println!("{}", opts().usage(&brief));
}



#[derive(Debug)]
pub struct Args {
    pub program_name: String,
    pub source_dir: String,
    pub target_dir: String,
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

        let (target_dir, source_dir, mut files) = match (args.free.len(), args.free) {
            (0, _) => return Err(Error::MissingTargetDir { program_name: program_name }),
            (1, _) => return Err(Error::MissingSourceDir { program_name: program_name }),
            (_, a) => (a[0].clone(), a[1].clone(), Vec::from(&a[2..])),
        };

        if files.len() == 0 {
            files = vec![source_dir.clone()];
        }

        Ok(Args {
            program_name: program_name,
            target_dir: target_dir,
            source_dir: source_dir,
            dry_run: dry_run,
            help: help,
            paths: files,
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
