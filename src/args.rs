use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    MissingProgramName,
    MissingInputs { program_name: String }
}
use self::Error::*;

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            MissingProgramName => "Missing program name (argv[0])",
            MissingInputs { .. } => "No inputs specified"
        }
    }
}
impl fmt::Display for Error { fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.description()) } }



#[derive(Debug)]
pub struct Args {
    pub program_name: String,
    pub input: Vec<String>
}

impl Args {
    pub fn from_iter<T: IntoIterator<Item=String>>(args: T) -> Result<Args, Error> {
        let mut args = args.into_iter();
        let program_name = match args.next() { Some(s) => s, None => return Err(MissingProgramName) };
        // let target_directory = match args.next() { Some(s) => s, None => return Err(MissingTargetDir { program_name: program_name }.into()) };

        let input: Vec<_> = args.collect();
        if input.len() == 0 {
            return Err(MissingInputs { program_name: program_name })
        }
        Ok(Args {
            program_name: program_name,
            input: input
        })
    }
    pub fn from_env() -> Result<Args, Error> {
        use std::env;
        Args::from_iter(env::args())
    }
    pub fn usage(program_name: String) -> String {
        return format!("{} path0 path1 path2", program_name)
    }
}


