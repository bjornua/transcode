use std::process::Command;
use rustc_serialize::json;
use std::error::Error as StdError;
use std::fmt;
use std::ffi::OsStr;
use std::io;

const FORMAT_WHITELIST: &'static [&'static str] =
    &["asf", "avi", "matroska,webm", "mov,mp4,m4a,3gp,3g2,mj2", "mpeg", "mpegts", "flv", "wav"];
const FORMAT_BLACKLIST: &'static [&'static str] =
    &["bmp_pipe", "gif", "image2", "jpeg_pipe", "lrc", "png_pipe", "tiff_pipe", "tty", "srt"];

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
            Error::RunError(_) => "An error happened trying to run ffprobe",
            Error::ParseError(_) => "An error happened while parsing the output from ffprobe",
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
pub enum ParseErrorKind {
    Json,
    Format,
    UnknownFormat(String),
    Duration,
    Stream,
    VideoCodecName,
    AudioCodecName,
    Height,
    Width,
    FPS,
}


#[derive(Debug)]
pub struct ParseError {
    input: String,
    kind: ParseErrorKind,
}

#[derive(Debug)]
pub enum RunError {
    OutputCaptureError(io::Error),
    StdOutUTF8Error(::std::string::FromUtf8Error),
    StdErrUTF8Error(::std::string::FromUtf8Error),
    Unsuccessful {
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    StdOutEmpty { stderr: String },
}

impl StdError for RunError {
    fn description(&self) -> &str {
        match *self {
            RunError::OutputCaptureError(_) => "An error happened while trying to capture output",
            RunError::StdOutUTF8Error(_) => "Could not parse stdout as UTF-8",
            RunError::StdErrUTF8Error(_) => "Could not parse stderr as UTF-8",
            RunError::Unsuccessful { .. } => "Exit code non zero or no exit code was returned",
            RunError::StdOutEmpty { .. } => "StdOut was empty",
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match *self {
            RunError::OutputCaptureError(ref e) => Some(e),
            RunError::StdOutUTF8Error(ref e) => Some(e),
            RunError::StdErrUTF8Error(ref e) => Some(e),
            RunError::Unsuccessful { .. } => None,
            RunError::StdOutEmpty { .. } => None,

        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::borrow::Cow;

        match *self {
            RunError::Unsuccessful { ref stderr, ref stdout, ref exit_code } => {
                let exit_code =
                    exit_code.map_or(Cow::Borrowed("None"), |x| Cow::Owned(x.to_string()));
                let stdout = stdout.trim();
                let stderr = stderr.trim();
                write!(f,
                       "{description}.\n\nExit Code: \
                        {exit_code}\n\nStdErr:\n{stderr}\n\nStdOut:\n{stdout}",
                       description = self.description(),
                       exit_code = exit_code,
                       stdout = stdout,
                       stderr = stderr)
            }
            RunError::StdOutEmpty { ref stderr } => {
                let stderr = stderr.trim();
                write!(f,
                       "{description}.\n\nStdErr:\n{stderr}",
                       description = self.description(),
                       stderr = stderr)
            }
            RunError::OutputCaptureError(_) |
            RunError::StdOutUTF8Error(_) |
            RunError::StdErrUTF8Error(_) => write!(f, "{}.", self.description()),
        }
    }
}


impl StdError for ParseError {
    fn description(&self) -> &str {
        use self::ParseErrorKind::*;
        match self.kind {
            Json => "Could not parse JSON",
            Format => "Could not get format name from JSON",
            UnknownFormat(_) => "Unrecognized format string",
            Duration => "Could not get duration from JSON",
            Stream => "Could not get streams from JSON",
            VideoCodecName => "Could not get codec name from video stream",
            AudioCodecName => "Could not get codec name from audio stream",
            Height => "Could not get height from video stream",
            Width => "Could not get width from video stream",
            FPS => "Could not get fps from video stream",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ParseErrorKind::UnknownFormat(ref format) => {
                write!(f,
                       "{}\nFormat: {:?}\nInput:\n{}",
                       self.description(),
                       format,
                       self.input.trim())
            }
            _ => write!(f, "{}\nInput:\n{}", self.description(), self.input.trim()),
        }
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


pub fn ffprobe<T: AsRef<OsStr>>(path: T) -> Result<Option<FFProbe>, Error> {
    let string = match ffprobe_run(path.as_ref()) {
        Ok(string) => string,
        Err(RunError::Unsuccessful { .. }) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    return ffprobe_parse(string).map_err(|e| e.into());
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

    match (result.status.success(), stdout.len() != 0) {
        (true, true) => Ok(stdout),
        (false, false) | (false, true) => {
            Err(RunError::Unsuccessful {
                exit_code: result.status.code(),
                stderr: stderr,
                stdout: stdout,
            })
        }
        (true, false) => Err(RunError::StdOutEmpty { stderr: stderr }),
    }
}


fn ffprobe_parse(text: String) -> Result<Option<FFProbe>, ParseError> {
    let json = match json::Json::from_str(&text).ok() {
        Some(c) => c,
        None => {
            return Err(ParseError {
                input: text,
                kind: ParseErrorKind::Json,
            })
        }
    };

    let format = match json.find_path(&["format", "format_name"]).and_then(|j| j.as_string()) {
        Some(f) => f,
        None => {
            return Err(ParseError {
                input: text,
                kind: ParseErrorKind::Format,
            })
        }
    };

    match (FORMAT_WHITELIST.contains(&format), FORMAT_BLACKLIST.contains(&format)) {
        (false, true) => {
            return Ok(None);
        }
        (true, false) => (),
        (true, true) | (false, false) => {
            return Err(ParseError {
                input: text,
                kind: ParseErrorKind::UnknownFormat(format.to_string()),
            });
        }
    }

    let duration: f64 = match json.find_path(&["format", "duration"]) {
        Some(j) => {
            match j.as_string().and_then(|s| s.parse::<f64>().ok()) {
                Some(s) => s,
                None => {
                    return Err(ParseError {
                        input: text,
                        kind: ParseErrorKind::Duration,
                    })
                }
            }
        }
        None => return Ok(None),
    };


    let streams = match json.find("streams").and_then(|j| j.as_array()) {
        Some(t) => t,
        None => {
            return Err(ParseError {
                input: text,
                kind: ParseErrorKind::Stream,
            })
        }
    };
    let audio_stream = streams.into_iter()
        .filter(|&x| x.find("codec_type").and_then(|x| x.as_string()) == Some("audio"))
        .filter_map(|x| x.as_object())
        .next();

    let audio: Option<Audio> = if let Some(stream) = audio_stream {
        let audio_codec = match stream.get("codec_name").and_then(|x| x.as_string()) {
            Some(t) => t.to_string(),
            None => {
                return Err(ParseError {
                    input: text,
                    kind: ParseErrorKind::AudioCodecName,
                })
            }
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
            None => {
                return Err(ParseError {
                    input: text,
                    kind: ParseErrorKind::VideoCodecName,
                })
            }
        };

        let height = match stream.get("height").and_then(|x| x.as_u64()) {
            Some(t) => t,
            None => {
                return Err(ParseError {
                    input: text,
                    kind: ParseErrorKind::Height,
                })
            }
        };

        let width = match stream.get("width").and_then(|x| x.as_u64()) {
            Some(t) => t,
            None => {
                return Err(ParseError {
                    input: text,
                    kind: ParseErrorKind::Width,
                })
            }
        };

        let fps: f64 = match stream.get("r_frame_rate")
            .and_then(|x| x.as_string())
            .and_then(|s| parse_fraction(&String::from(s))) {
            Some(t) => t,
            None => {
                return Err(ParseError {
                    input: text,
                    kind: ParseErrorKind::FPS,
                })
            }
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

    Ok(Some(FFProbe {
        duration: duration,
        audio: audio,
        video: video,
    }))
}

fn parse_fraction(t: &String) -> Option<f64> {
    let mut split = t.split("/").map(|n| n.parse::<u64>());

    match (split.next(), split.next()) {
        (_, Some(Ok(0))) => None,
        (Some(Ok(p)), Some(Ok(q))) => Some((p as f64) / (q as f64)),
        (_, _) => return None,
    }
}
