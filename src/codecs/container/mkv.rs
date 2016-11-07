use super::super::{Codec as CodecTrait, Error};
use super::super::audio;
use super::super::video;

use std::ffi::OsString;

#[derive(Clone, Debug)]
pub struct Codec {
    video: video::Codec,
    audio: audio::Codec,
}

impl Default for Codec {
    fn default() -> Self {
        Codec {
            video: video::Codec::default(),
            audio: audio::Codec::default(),
        }
    }
}

impl CodecTrait for Codec {
    fn from_args<'a, T: Iterator<Item = &'a str>>(args: T) -> Result<(Self, T), Error> {
        let (video, args) = try!(video::Codec::from_args(args));
        let (audio, args) = try!(audio::Codec::from_args(args));

        return Ok((Codec { video: video, audio: audio }, args))

    }
    fn to_ffmpeg_args(&self) -> Vec<OsString> {
        ["-f", "matroska"].into_iter().map(|&s| OsString::from(s))
            .chain(self.video.to_ffmpeg_args())
            .chain(self.audio.to_ffmpeg_args())
            .collect()
    }

    fn to_ffprobe_id(&self) -> (Option<&'static str>, Option<&'static str>) {
        ((self.video.to_ffprobe_id().0, self.audio.to_ffprobe_id().1))
    }
    fn to_examples() -> Vec<Vec<&'static str>> {
        let (audio_example, video_example) = match (audio::Codec::to_examples().into_iter().next(),
                                                    video::Codec::to_examples().into_iter().next()) {
            (Some(a), Some(v)) => (a, v),
            _ => return vec![],
        };
        let res: Vec<&'static str> = video_example.into_iter()
            .chain(audio_example)
            .collect();

        return vec![res];
    }
}
