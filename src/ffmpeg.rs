use conversion::Conversion;
use std::process::Command;
use std::process::Stdio;
use std::ffi::OsStr;
use std::io::Read;

pub fn ffmpeg(con: &Conversion) {
    let mut c = Command::new("ffmpeg");
    c.args(&[
        OsStr::new("-i"),       OsStr::new(&con.source.path),
        OsStr::new("-f"),       OsStr::new("matroska"),
        OsStr::new("-c:v"),     OsStr::new("libx264"),
        OsStr::new("-level"),   OsStr::new("4.1"),
        OsStr::new("-preset"),  OsStr::new("placebo"),
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
    let mut s = String::new();


    child.wait();
    child.stderr.unwrap().read_to_string(&mut s);

    println!("{}", s);

    // let result = c.output().unwrap();
    // let stdout = String::from_utf8(result.stdout).unwrap();
    // let stderr = String::from_utf8(result.stderr).unwrap();
    // let status = result.status.code().unwrap();

    // println!("{}", stderr);
}

