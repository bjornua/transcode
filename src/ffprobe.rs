use std::process::Command;
use rustc_serialize::json;
use std::error::Error as StdError;
use std::fmt;
use std::ffi::OsStr;

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

#[derive(Debug,Clone,PartialEq)]
pub enum Error {
    JsonError { input: String },
    DurationError { input: String },
    StreamError { input: String },
    VideoCodecNameError { input: String },
    AudioCodecNameError { input: String },
    HeightError { input: String },
    WidthError { input: String },
    FPSError { input: String },
    RunError { output: String },
}

use self::Error::*;


impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            JsonError { .. } => "Could not parse JSON",
            DurationError { .. } => "Could not get duration from JSON",
            StreamError { .. } => "Could not get streams from JSON",
            VideoCodecNameError { .. } => "Could not get codec name from video stream",
            AudioCodecNameError { .. } => "Could not get codec name from audio stream",
            HeightError { .. } => "Could not get height from video stream",
            WidthError { .. } => "Could not get width from video stream",
            FPSError { .. } => "Could not get fps from video stream",
            RunError { .. } => "Running \"ffprobe\" failed",
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub fn ffprobe<T: AsRef<OsStr>>(path: T) -> Result<FFProbe, Error> {
    let text = match ffprobe_run(path.as_ref()) {
        Err(e) => return Err(e),
        Ok(t) => t,
    };

    return ffprobe_parse(text);

}

fn ffprobe_run(path: &OsStr) -> Result<String, Error> {
    let mut c = Command::new("ffprobe");
    c.args(&[OsStr::new("-print_format"),
             OsStr::new("json"),
             OsStr::new("-hide_banner"),
             OsStr::new("-loglevel"),
             OsStr::new("error"),
             OsStr::new("-show_streams"),
             OsStr::new("-show_format"),
             path]);

    let result = c.output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap();
    let status = result.status.code().unwrap();

    match (status, stderr.len()) {
        (0, 0) => Ok(stdout),
        (_, 0) => Err(RunError { output: stdout.trim().to_string() }),
        (_, _) => Err(RunError { output: stderr.trim().to_string() }),
    }
}

fn ffprobe_parse(text: String) -> Result<FFProbe, Error> {
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
