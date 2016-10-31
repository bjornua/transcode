# transcode
[![Cargo version][cargo-image]][cargo-url]

Command-line utility for converting directories of audio/video format to run on a raspberry pi.

## Example
```bash
transcode ~/Videos/Family ~/Videos/Vacation
```
* Creates directory: `~/Videos - Converted/`
* Converts video/audio files in `~/Videos/Family` to `~/Videos - Converted/Family`
* Converts video/audio files in `~/Videos/Vacation` to `~/Videos - Converted/Vacation`

## Features
* Shows progress for individual files and total progress
* Shows ETA for individual files and total ETA
* Takes directory as input, automatically identify audio/video files within.
* Copies files that are already in the target format instead of processing.

## Formats
Currently the only target format is hardcoded and is:

| Container | Audio     | Video     |
|-----------|-----------|-----------|
|MKV        | Opus 192k | h.264 4.1 |

## Installation
* Install rust (https://www.rust-lang.org/en-US/downloads.html)
* Install ffmpeg (https://ffmpeg.org/download.html)
* Run `cargo install transcode`
* Run `~/.cargo/bin/transcode` (you can add `~/.cargo/bin/` to `PATH`)

## Motivation
I ran into the problem of having many video files of various formats that needed to run on a raspberry pi.
So i needed a script to convert all of them for playback in a format that raspberry pi can run.

You could of course just loop through all the files. Though i found the following issues:

Problems:
* There is no ETA on when the script will finish.
* Files would be scattered in the file system.

So i decided i wanted to write a more specific program for the job.

[cargo-image]: https://img.shields.io/crates/v/transcode.svg
[cargo-url]: https://crates.io/crates/transcode
