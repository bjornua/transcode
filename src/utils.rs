pub fn repeat_str<T: AsRef<str>>(s: T, times: usize) -> String {
    s.as_ref().chars().cycle().take(times).collect()
}


pub fn erase_up(lines: usize) {
    for _ in 0..lines {
        print!("\x1B[2K\x1B[A");
    }
    print!("\r");
}
