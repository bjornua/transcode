use conversion::Conversion;
use std::process::Command;
use std::process::Stdio;
use std::ffi::OsStr;
use std::thread;
use regexreader::RegexReadIterator;

pub fn ffmpeg(con: &Conversion) {
    let mut c = Command::new("ffmpeg");
    c.args(&[
        OsStr::new("-i"),       OsStr::new(&con.source.path),
        OsStr::new("-f"),       OsStr::new("matroska"),
        OsStr::new("-c:v"),     OsStr::new("libx264"),
        OsStr::new("-level"),   OsStr::new("4.1"),
        OsStr::new("-preset"),  OsStr::new("medium"),
        OsStr::new("-crf"),     OsStr::new("18"),
        OsStr::new("-c:a"),     OsStr::new("opus"),
        OsStr::new("-b:a"),     OsStr::new("192k"),
        OsStr::new("-y"),
        OsStr::new("/dev/null") // c.target.path
    ]);
    c.stderr(Stdio::piped());
    c.stdout(Stdio::null());
    c.stdin(Stdio::null());

    let mut child = c.spawn().unwrap();
    if let Some(mut stderr) = child.stderr {
        child.stderr = Some(thread::spawn(move || {
            {
                let matches = RegexReadIterator::new("time=[0-9:]+", &mut stderr);
                if let Ok(matches) = matches {
                    for m in matches {
                        println!("{:?}", m)
                    }
                }
            }
            stderr
        }).join().unwrap());
    };

    let _status = child.wait();
}
