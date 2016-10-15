use args;
use ffprobe;
use main;
use source;




pub fn print_error(k: main::ErrorKind) {
    use main::ErrorKind::*;
    println!("\n-------------------- Error --------------------");
    match k {
        ArgError(e) => print_arg_error(e),
        SourceError(e) => print_source_error(e)
    }
    println!("-----------------------------------------------");
}

fn print_source_error(source::Error {kind, path}: source::Error) {
    use source::ErrorKind::*;
    println!("Path: {:?}", path);
    match kind {
        FFProbeError {error: e} =>print_ffprobe_error(e),
        PathError { error: msg } => {
            println!("Error: Path failed ({})", msg);

        }
    };
}


fn print_ffprobe_error(ffprobe::Error {kind, msg}: ffprobe::Error) {
    use ffprobe::ErrorKind::*;
    println!("Error: ffprobe failed ({})", msg);
    match kind {
        RunError { output } => {
            println!("ffprobe output:\n\n{}\n", output)
        },
        JsonError { .. } => (),
        DurationError { .. } => (),
        StreamError { .. } => (),
        VideoStreamError { .. } => (),
        HeightError { .. } => (),
        WidthError { .. } => (),
        FPSError { .. } => (),
    }

}

fn print_arg_error(args::Error {kind, msg}: args::Error) {
    use args::ErrorKind::*;
    println!("Error: Argument failure ({})", msg);
    match kind {
        MissingProgramName => {
            ()
        },
        MissingTargetDir { program_name } | MissingInputs { program_name } => {
            println!("");
            println!("Usage: {}", args::Args::usage(program_name))
        },
    }
}

