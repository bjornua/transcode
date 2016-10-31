# transcode
Command-line utility for converting directories of audio/video format to run on a raspberry pi.

## Example
```bash
transcode ~/Videos/Family ~/Videos/Vacation
```
Converts video/audio files (mkv, avi, mp4, ...) to .mkv

## Features
* Shows %Done for individual files and total %Done
* Shows ETA for individual files and total ETA
* Takes directory as input, automatically identify audio/video files.
* Copies files that are already in the target format instead of processing.

## Limitations
Currently the only target format is hardcoded and is:

| Container | Audio     | Video     |
|-----------|-----------|-----------|
|MKV        | Opus 192k | h.264 4.1 |

## Screenshot


## Motivation
I ran into the problem of having many video files of various formats that needed to run on a raspberry pi.
So i needed a script to convert all of them for playback in a format that raspberry pi can run.

You could of course just loop through all the files. Though i found the following issues:

Problems:
* There is no ETA on when the script will finish.
* 

Also i wanted to try out rust.

## Future
* Almost everything is hardcoded. Needs more configuration options.






Tested to compile under windows
