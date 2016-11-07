use std::ffi::{OsString};
use super::super::{Codec as CodecTrait, Error};

#[derive(Clone, Debug)]
pub struct Codec {
    bitrate: u64
}

impl Default for Codec {
    fn default() -> Self {
        Codec { bitrate: 192 }
    }
}

impl CodecTrait for Codec {
    fn from_args<'a, T: Iterator<Item = &'a str>>(mut args: T) -> Result<(Self, T), Error> {
        let bitrate = match args.next() {
            Some(s) => s,
            None => return Err(Error::TooShort)
        };

        let bitrate = match bitrate.parse::<i64>() {
            Ok(s) if s >=6 && s <= 255 => s as u64,
            Ok(_) => return Err(Error::InvalidArg(bitrate.to_string(), "Bitrate must be between 6 and 255")),
            Err(_) => return Err(Error::InvalidArg(bitrate.to_string(), "Bitrate must be a number"))
        };
        Ok((Codec { bitrate: bitrate }, args))
    }
    fn to_ffmpeg_args<'a>(&self) -> Vec<OsString> {
        vec![
            "-c:a".into(),
            "opus".into(),
            "-b:a".into(),
            format!("{}k", self.bitrate).into()
        ]
    }
    fn to_ffprobe_id(&self) -> (Option<&'static str>, Option<&'static str>) {
        (None, Some("opus"))
    }
    fn to_examples() -> Vec<Vec<&'static str>> {
        vec![vec!["192"]]

    }
}


