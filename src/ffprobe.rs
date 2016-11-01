use std::process::Command;
use rustc_serialize::json;
use std::error::Error as StdError;
use std::fmt;
use std::ffi::OsStr;
use std::io;

#[derive(Debug,Clone,PartialEq)]
pub struct Video {
    pub width: u64,
    pub height: u64,
    pub fps: f64,
    pub codec: String,
}
#[derive(Debug,Clone,PartialEq)]
pub struct Audio {
    pub codec: String,
}

#[derive(Debug,Clone,PartialEq)]
pub struct FFProbe {
    pub duration: f64,
    pub video: Option<Video>,
    pub audio: Option<Audio>,
}

impl FFProbe {
    pub fn mpixel(&self) -> f64 {
        if let Some(ref video) = self.video {
            let per_frame = video.width * video.height;
            let frames = self.duration * video.fps;
            return ((per_frame as f64) * frames) / 1000000.;
        } else {
            0.
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ParseError(ParseError),
    RunError(RunError),
}
impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::RunError(_) => "An error happened while running ffprobe",
            Error::ParseError(_) => "An error happedn while parsing the output from ffprobe",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::RunError(ref e) => Some(e),
            Error::ParseError(ref e) => Some(e),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}


#[derive(Debug)]
pub enum ParseError {
    JsonError { input: String },
    DurationError { input: String },
    StreamError { input: String },
    VideoCodecNameError { input: String },
    AudioCodecNameError { input: String },
    HeightError { input: String },
    WidthError { input: String },
    FPSError { input: String },
    RunError(RunError),
}

#[derive(Debug)]
pub enum RunError {
    OutputCaptureError(io::Error),
    StdOutUTF8Error(::std::string::FromUtf8Error),
    StdErrUTF8Error(::std::string::FromUtf8Error),
    OutputError {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
}

impl StdError for RunError {
    fn description(&self) -> &str {
        match *self {
            RunError::OutputCaptureError(_) => "An error happened while trying to capture output",
            RunError::StdOutUTF8Error(_) => "Could not parse stdout as UTF-8",
            RunError::StdErrUTF8Error(_) => "Could not parse stderr as UTF-8",
            RunError::OutputError { .. } => "Exit code was non zero or stderr was not empty",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            RunError::OutputCaptureError(ref e) => Some(e),
            RunError::StdOutUTF8Error(ref e) => Some(e),
            RunError::StdErrUTF8Error(ref e) => Some(e),
            RunError::OutputError { .. } => None,

        }
    }
}
impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        use self::ParseError::*;
        match *self {
            JsonError { .. } => "Could not parse JSON",
            DurationError { .. } => "Could not get duration from JSON",
            StreamError { .. } => "Could not get streams from JSON",
            VideoCodecNameError { .. } => "Could not get codec name from video stream",
            AudioCodecNameError { .. } => "Could not get codec name from audio stream",
            HeightError { .. } => "Could not get height from video stream",
            WidthError { .. } => "Could not get width from video stream",
            FPSError { .. } => "Could not get fps from video stream",
            RunError(_) => "Running \"ffprobe\" failed",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseError(err)
    }
}
impl From<RunError> for Error {
    fn from(err: RunError) -> Self {
        Error::RunError(err)
    }
}


pub fn ffprobe<T: AsRef<OsStr>>(path: T) -> Result<FFProbe, Error> {
    let text = try!(ffprobe_run(path.as_ref()));
    return ffprobe_parse(text).map_err(|e| e.into());

}

fn ffprobe_run(path: &OsStr) -> Result<String, RunError> {
    let mut c = Command::new("ffprobe");
    c.args(&[OsStr::new("-print_format"),
             OsStr::new("json"),
             OsStr::new("-hide_banner"),
             OsStr::new("-loglevel"),
             OsStr::new("error"),
             OsStr::new("-show_streams"),
             OsStr::new("-show_format"),
             path]);

    let result = try!(c.output().map_err(|e| RunError::OutputCaptureError(e)));
    let stdout = try!(String::from_utf8(result.stdout).map_err(|e| RunError::StdOutUTF8Error(e)));
    let stderr = try!(String::from_utf8(result.stderr).map_err(|e| RunError::StdErrUTF8Error(e)));

    match (result.status.success(), stderr.len()) {
        (true, 0) => Ok(stdout),
        (_, _) => {
            Err(RunError::OutputError {
                exit_code: result.status.code(),
                stderr: stderr,
                stdout: stdout,
            })
        }
    }
}


fn ffprobe_parse(text: String) -> Result<FFProbe, ParseError> {
    use self::ParseError::*;
    let json = match json::Json::from_str(&text).ok() {
        Some(c) => c,
        None => return Err(JsonError { input: text }),
    };

    let streams = match json.find("streams").and_then(|j| j.as_array()) {
        Some(t) => t,
        None => return Err(StreamError { input: text }),
    };
    let audio_stream = streams.into_iter()
        .filter(|&x| x.find("codec_type").and_then(|x| x.as_string()) == Some("audio"))
        .filter_map(|x| x.as_object())
        .next();

    let audio: Option<Audio> = if let Some(stream) = audio_stream {
        let audio_codec = match stream.get("codec_name").and_then(|x| x.as_string()) {
            Some(t) => t.to_string(),
            None => return Err(AudioCodecNameError { input: text }),
        };
        Some(Audio { codec: audio_codec })
    } else {
        None
    };

    let video_stream = streams.into_iter()
        .filter(|&x| x.find("codec_type").and_then(|x| x.as_string()) == Some("video"))
        .filter_map(|x| x.as_object())
        .next();

    let video: Option<Video> = if let Some(stream) = video_stream {
        let video_codec = match stream.get("codec_name").and_then(|x| x.as_string()) {
            Some(t) => t.to_string(),
            None => return Err(VideoCodecNameError { input: text }),
        };

        let height = match stream.get("height").and_then(|x| x.as_u64()) {
            Some(t) => t,
            None => return Err(HeightError { input: text }),
        };

        let width = match stream.get("width").and_then(|x| x.as_u64()) {
            Some(t) => t,
            None => return Err(WidthError { input: text }),
        };

        let fps: f64 = match stream.get("avg_frame_rate")
            .and_then(|x| x.as_string())
            .and_then(|s| parse_fraction(&String::from(s))) {
            Some(t) => t,
            None => return Err(FPSError { input: text }),
        };

        Some(Video {
            codec: video_codec,
            width: width,
            height: height,
            fps: fps,
        })
    } else {
        None
    };

    let duration: f64 = match json.find_path(&["format", "duration"])
        .and_then(|j| j.as_string())
        .and_then(|s| s.parse::<f64>().ok()) {
        Some(t) => t,
        None => return Err(DurationError { input: text }),
    };

    Ok(FFProbe {
        duration: duration,
        audio: audio,
        video: video,
    })
}

fn parse_fraction(t: &String) -> Option<f64> {
    let mut split = t.split("/").map(|n| n.parse::<u64>());

    match (split.next(), split.next()) {
        (_, Some(Ok(0))) => None,
        (Some(Ok(p)), Some(Ok(q))) => Some((p as f64) / (q as f64)),
        (_, _) => return None,
    }
}
