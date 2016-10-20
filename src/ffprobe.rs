
use std::process::Command;
use rustc_serialize::json;



#[derive(Debug)]
pub enum ErrorKind {
    JsonError {input: String},
    DurationError {input: String},
    StreamError {input: String},
    VideoStreamError { input: String },
    HeightError {input: String},
    WidthError {input: String},
    FPSError {input: String},
    RunError {output: String}
}

use self::ErrorKind::*;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub msg: String
}

#[derive(Debug)]
pub struct FFProbe {
    pub width: u64,
    pub height: u64,
    pub duration: f64,
    pub fps: f64
}

impl FFProbe {
    pub fn mpixel(&self) -> f64 {
        let per_frame = self.width * self.height;
        let frames = self.duration * self.fps;
        return ((per_frame as f64) * frames) / 1000000.;
    }
}


impl Error {
    fn new(kind: ErrorKind) -> Error {
        let msg = match kind {
            JsonError {input: _} => "Could not parse JSON",
            DurationError {input: _} => "Could not get duration from JSON",
            StreamError {input: _} => "Could not get streams from JSON",
            VideoStreamError {input: _} => "Could not get a find a video stream",
            HeightError {input: _} => "Could not get height from video stream",
            WidthError {input: _} => "Could not get width from video stream",
            FPSError {input: _} => "Could not get fps from video stream",
            RunError {output: _} => "Running \"ffprobe\" failed"
        };
        Error{kind: kind, msg: msg.to_string()}
    }
}


pub fn ffprobe(path: &str) -> Result<FFProbe, Error> {
    let text = match ffprobe_run(&path) {
        Err(e) => return Err(e),
        Ok(t) => t
    };

    return ffprobe_parse(text)

}

fn ffprobe_parse(text: String) -> Result<FFProbe, Error> {
    let json = match json::Json::from_str(&text).ok() {
        Some(c) => c,
        None => return Err(Error::new(JsonError{ input: text })),
    };

    let streams = match json.find("streams").and_then(|j| j.as_array()) {
        Some(t) => t,
        None => return Err(Error::new(StreamError{ input: text }))
    };

    let video_stream = match streams.into_iter()
        .filter(|&x| x.find("codec_type").and_then(|x| x.as_string()) == Some("video"))
        .filter_map(|x| x.as_object()).next() {
        Some(t) => t,
        None => return Err(Error::new(VideoStreamError { input: text}))
    };

    let height = match video_stream.get("height").and_then(|x| x.as_u64()) {
        Some(t) => t,
        None => return Err(Error::new(HeightError { input: text } ))
    };

    let width = match video_stream.get("width").and_then(|x| x.as_u64()) {
        Some(t) => t,
        None => return Err(Error::new(WidthError { input: text } ))
    };

    let fps: f64 = match video_stream.get("avg_frame_rate")
        .and_then(|x| x.as_string())
        .and_then(|s| parse_fraction(&String::from(s))) {
        Some(t) => t,
        None => return Err(Error::new(FPSError { input: text } ))
    };

    let duration: f64 = match json.find_path(&["format", "duration"])
        .and_then(|j| j.as_string())
        .and_then(|s| s.parse::<f64>().ok()) {
        Some(t) => t,
        None => return Err(Error::new(DurationError{ input: text }))
    };

    Ok(FFProbe {
        width: width,
        height: height,
        duration: duration,
        fps: fps
    })
}


fn ffprobe_run(path: &str) -> Result<String, Error> {
    let mut c = Command::new("ffprobe");
    c.args(&[
        "-print_format", "json",
        "-hide_banner",
        "-loglevel", "error",
        "-show_streams",
        "-show_format",
        path
    ]);

    let result = c.output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    let stderr = String::from_utf8(result.stderr).unwrap();
    let status = result.status.code().unwrap();

    match (status, stderr.len()) {
        (0, 0) => Ok(stdout),
        (_, 0) => Err(Error::new(RunError{output: stdout.trim().to_string()})),
        (_, _) => Err(Error::new(RunError{output: stderr.trim().to_string()}))
    }
}
fn parse_fraction (t: &String) -> Option <f64> {
    let mut split = t.split("/").map(|n| n.parse::<u64>() );

    match (split.next(), split.next()) {
        (_, Some(Ok(0))) => None,
        (Some(Ok(p)), Some(Ok(q))) => Some((p as f64) / (q as f64)),
        (_, _) => return None
    }
}
