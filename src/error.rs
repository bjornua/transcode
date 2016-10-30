use args;
use ffmpeg;
// use ffprobe;
use main;
// use source;
use std::error::Error as StdError;

pub fn print_error(k: main::Error) {
    use main::Error::*;
    println!("\n-------------------- Error --------------------");
    match k {
        ArgError(e) => print_arg_error(e),
        // SourceError(e) => print_source_error(e),
        FFmpegError(e) => print_ffmpeg_error(e)
    }
    println!("-----------------------------------------------");
}


// fn print_source_error(kind: source::Error) {
//     use source::Error::*;
//     match kind {
//         FFProbeError {error: e, ..} => print_ffprobe_error(e),
//         PathError { error: e, .. } => {
//             println!("Error: Path failed ({})", e.description());
//         }
//     };
// }


// fn print_ffprobe_error(kind: ffprobe::Error) {
//     use ffprobe::Error::*;
//     println!("Error: ffprobe failed ({})", kind.description());
//     match kind {
//         RunError { output } => {
//             println!("ffprobe output:\n\n{}\n", output)
//         },
//         JsonError { .. } => (),
//         DurationError { .. } => (),
//         StreamError { .. } => (),
//         HeightError { .. } => (),
//         WidthError { .. } => (),
//         FPSError { .. } => (),
//         VideoCodecNameError { .. } => (),
//         AudioCodecNameError { .. } => (),
//     }

// }

fn print_arg_error(kind: args::Error) {
    use args::Error::*;
    println!("Error: Argument failure ({})", kind.description());;
    match kind {
        MissingProgramName => {
            ()
        },
        /*MissingTargetDir { program_name } | */MissingInputs { program_name/*, target_directory: _ */} => {
            println!("");
            println!("Usage: {}", args::Args::usage(program_name))
        },
    }
}

fn print_ffmpeg_error(err: ffmpeg::Error) {
    println!("Error: FFmpeg failure ({})", err.description());
    match err {
        ffmpeg::Error::RunError {stdout, stderr} => {
            println!("FFmpeg stderr:\n{}\n\nFFmpeg stdout:\n{}", stderr.trim(), stdout.trim())
        },
        ffmpeg::Error::IO(_) => (),
        ffmpeg::Error::NoStderr => ()
    }
}
