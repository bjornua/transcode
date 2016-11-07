mod h264;

use super::{Codec as CodecTrait, Error};
use std::ffi;
use std::iter::once;


#[derive(Clone, Debug)]
pub enum Codec {
    H264(h264::Codec),
}

impl Default for Codec {
    fn default() -> Self {
        Codec::H264(h264::Codec::default())
    }
}


impl CodecTrait for Codec {
    fn from_args<'a, T: Iterator<Item = &'a str>>(mut args: T) -> Result<(Self, T), Error> {
        let name = match args.next() {
            Some(s) => s,
            None => return Err(Error::TooShort),

        };

        let (codec, args) = match name {
            "h264" => {
                let (codec, args) = try!(h264::Codec::from_args(args));
                (Codec::H264(codec), args)
            },
            _ => return Err(Error::InvalidArg(name.to_string(), "Unsupported audio codec"))
        };

        return Ok((codec, args))
    }
    fn to_ffmpeg_args(&self) -> Vec<ffi::OsString> {
        match *self {
            Codec::H264(ref c) => c.to_ffmpeg_args()
        }
    }
    fn to_ffprobe_id(&self) -> (Option<&'static str>, Option<&'static str>) {
        match *self {
            Codec::H264(ref c) => c.to_ffprobe_id()
        }


    }
    fn to_examples() -> Vec<Vec<&'static str>> {
        once(
            ("h264", h264::Codec::to_examples())
        ).into_iter().flat_map(|(codec_name, examples)| {
            examples.into_iter().map(move |example| {
                once(codec_name).chain(example).collect::<Vec<_>>()
            })
        }).collect()
    }
}

