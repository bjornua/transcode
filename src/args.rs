use std::fmt;
use std::error::Error as StdError;
use getopts::{self, Options};
use codecs;
use codecs::Codec;

#[derive(Debug)]
pub enum Error {
    MissingProgramName,
    MissingTargetDir { program_name: String },
    MissingSourceDir { program_name: String },
    GetOptsFail {
        program_name: String,
        error: getopts::Fail,
    },
    Help { program_name: String },
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::MissingProgramName => "Missing program name (argv[0])",
            Error::MissingTargetDir { .. } => "No OUTPUT_DIRECTORY specified",
            Error::MissingSourceDir { .. } => "No INPUT_DIRECTORY specified",
            Error::GetOptsFail { .. } => "Argument error",
            Error::Help { .. } => "Help specified",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::MissingProgramName => None,
            Error::Help { .. } => None,
            Error::MissingTargetDir { .. } => None,
            Error::MissingSourceDir { .. } => None,
            Error::GetOptsFail { ref error, .. } => Some(error),
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
    opts.optflag("f", "format", "Set the output format");
    opts
}

pub fn print_usage(program_name: &str) {
    let brief = format!("Usage: {} [OPTION]... INPUT_DIRECTORY OUTPUT_DIRECTORY [INPUT_FILE]...",
                        program_name);
    print!("{}", opts().usage(&brief));
    println!("");
    println!("Examples of the --format option:");
    for example in codecs::container::Codec::to_examples() {
        println!("    --format={}", example.join(","))
    }
}



#[derive(Debug)]
pub struct Args {
    pub program_name: String,
    pub source_dir: String,
    pub target_dir: String,
    pub paths: Vec<String>,
    pub dry_run: bool,
    pub format: Option<String>,
}

impl Args {
    pub fn from_iter<T: IntoIterator<Item = String>>(args: T) -> Result<Args, Error> {
        let mut args = args.into_iter();

        let program_name = match args.next() {
            Some(s) => s.clone(),
            None => return Err(Error::MissingProgramName),
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
        if args.opt_present("help") {
            return Err(Error::Help { program_name: program_name });
        }

        let dry_run = args.opt_present("dry-run");
        let format = args.opt_str("format");

        let (source_dir, target_dir, mut files) = match (args.free.len(), args.free) {
            (0, _) => return Err(Error::MissingSourceDir { program_name: program_name }),
            (1, _) => return Err(Error::MissingTargetDir { program_name: program_name }),
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
            paths: files,
            format: format,
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
