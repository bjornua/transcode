use std::ffi::{OsString};
use super::super::{Codec as CodecTrait, Error};

#[derive(Clone, Debug)]
pub struct Codec {
    crf: u64,
    speed: &'static str
}

impl Default for Codec {
    fn default() -> Self {
        Codec { crf: 18, speed: SPEEDS[5] }
    }
}


/*
        if let Some(ref video) = con.source.ffprobe.video {
            if video.codec == "h264" {
                // Already the right codec, just copy
                args.extend(&[OsStr::new("-c:v"), OsStr::new("copy")]);
            }
        }

*/

const SPEEDS: [&'static str; 10] = [
    "ultrafast",
    "superfast",
    "veryfast",
    "faster",
    "fast",
    "medium",
    "slow",
    "slower",
    "veryslow",
    "placebo",
];
fn translate_speed<'a>(s: &'a str) -> Option<&'static str> {
    for &speed in &SPEEDS {
        if speed == s {
            return Some(speed)
        }
    }
    None
}

impl CodecTrait for Codec {
    fn from_args<'a, T: Iterator<Item = &'a str>>(mut args: T) -> Result<(Self, T), Error> {
        let (crf, speed) = match (args.next(),args.next(),) {
            (Some(crf),Some(speed)) => (crf, speed),
            _ => return Err(Error::TooShort)
        };

        let crf = match crf.parse::<i64>() {
            Ok(s) if s >= 0 && s <= 51 => s as u64,
            Ok(_) => return Err(Error::InvalidArg(crf.to_string(), "CRF must be between 0 and 51")),
            Err(_) => return Err(Error::InvalidArg(crf.to_string(), "CRF must be a number"))
        };
        let speed = match translate_speed(speed) {
            Some(s) => s,
            None => return Err(Error::InvalidArg(crf.to_string(), "Speed must be "))
        };

        Ok((Codec { crf: crf, speed: speed }, args))
    }
    fn to_ffmpeg_args<'a>(&self) -> Vec<OsString> {
        vec![
            "-c:v".into(),
            "libx264".into(),
            "-level".into(),
            "4.1".into(),
            "-preset".into(),
            self.speed.into(),
            "-crf".into(),
            format!("{}", self.crf).into(),
        ]
    }
    fn to_ffprobe_id(&self) -> (Option<&'static str>, Option<&'static str>) {
        (None, Some("h264"))
    }
    fn to_examples() -> Vec<Vec<&'static str>> {
        vec![vec!["18", "normal"]]

    }
}
