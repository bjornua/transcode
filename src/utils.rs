pub fn repeat_str<T: AsRef<str>>(s: T, times: usize) -> String {
    s.as_ref().chars().cycle().take(times).collect()
}
