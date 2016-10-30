
pub fn repeat_str<T: AsRef<str>>(s: T, times: usize) -> String {
    s.as_ref().chars().cycle().take(times).collect()
}

pub fn move_up() {
    print!("\x1B[A");
}

pub fn delete_line() {
    print!("\x1B[2K");
}

pub fn erase_up(lines: usize) {
    delete_line();
    for _ in 0..lines {
        move_up();
        delete_line();
    }
    print!("\r");
}

pub fn common_prefix<'a, T: PartialEq>(a: &'a[T], b: &[T]) -> &'a[T] {
    let common = a.into_iter().zip(b.into_iter()).take_while(
        |&(a, b)| a == b
    );

    return &a[0..common.count()]
}
