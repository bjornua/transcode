

#[derive(Debug)]
pub enum ErrorKind {
    MissingProgramName,
    MissingTargetDir { program_name: String },
    MissingInputs { program_name: String }
}



#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        use self::ErrorKind::*;

        let msg = match kind {
            MissingProgramName => "Missing program name (argv[0])",
            MissingTargetDir { .. } => "Missing target directory",
            MissingInputs { .. } => "No inputs specified"
        };
        Error{kind: kind, msg: msg.to_string()}
    }
}

pub struct Args {
    pub program_name: String,
    pub target_directory: String,
    pub input: Vec<String>
}

impl Args {
    pub fn from_iter<T: IntoIterator<Item=String>>(args: T) -> Result<Args, Error> {
        use self::ErrorKind::*;

        let mut args = args.into_iter();
        let program_name     = match args.next() { Some(s) => s, None => return Err(MissingProgramName.into()) };
        let target_directory = match args.next() { Some(s) => s, None => return Err(MissingTargetDir { program_name: program_name }.into()) };
        let input: Vec<_> = args.collect();
        if input.len() == 0 {
            return Err(MissingInputs { program_name: program_name }.into())
        }
        Ok(Args {
            program_name: program_name,
            target_directory: target_directory,
            input: input
        })
    }
    pub fn from_env() -> Result<Args, Error> {
        use std::env;
        Args::from_iter(env::args())
    }
    pub fn usage(program_name: String) -> String {
        return format!("{} target_directory file0 [file1 [file2 ...]]", program_name)
    }
}


