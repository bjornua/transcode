use constants;


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


pub fn common_prefix<'a, T: PartialEq>(a: &'a [T], b: &[T]) -> &'a [T] {
    let common = a.into_iter().zip(b.into_iter()).take_while(|&(a, b)| a == b);

    return &a[0..common.count()];
}


pub fn prompt<F: Fn(&str) -> bool>(question: &str, validator: F) -> Option<String> {
    use std::io::{self, Write, BufRead, BufReader};
    let mut lines = BufReader::new(io::stdin()).lines();

    loop {
        print!("{}: ", question);
        let _ = io::stdout().flush();
        match lines.next().and_then(|x| x.ok()) {
            Some(line) => {
                if validator(line.as_ref()) {
                    return Some(line);
                }
            }
            None => break,
        };
    }
    None
}

pub fn prompt_continue() -> bool {
    prompt("Do you want to continue [y/n]?", |s| s == "y" || s == "n").map_or(false, |s| s == "y")
}

const HEX: &'static[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'];

fn u64_to_hex(n: u64) -> String {
    let mut m: u64 = n.clone();
    let mut result: Vec<char> = Vec::with_capacity(16);
    loop {
        result.push(HEX[(m & 15) as usize]);
        println!("{}", m);

        if m < 16 {
            break;
        }
        m >>= 4
    }
    result.into_iter().rev().collect()
}

use std::hash::{Hash, Hasher, SipHasher};

pub fn hash_to_hex<T: Hash>(object: T) -> String {
    let mut hasher = SipHasher::new_with_keys(constants::SIP_KEY.0, constants::SIP_KEY.1);
    object.hash(&mut hasher);
    return u64_to_hex(hasher.finish())
}



#[test]
fn test_hex() {
    // println!("{}", u64_to_hex(14391273941934));
    let cases: &[(u64, &str)] = &[
                    (0x0, "0"),
                    (0x1, "1"),
                    (0x2, "2"),
                    (0x3, "3"),
                    (0x4, "4"),
                    (0x5, "5"),
                    (0x6, "6"),
                    (0x7, "7"),
                    (0x8, "8"),
                    (0x9, "9"),
                    (0xa, "a"),
                    (0xb, "b"),
                    (0xc, "c"),
                    (0xd, "d"),
                    (0xe, "e"),
                    (0xf, "f"),
                    (0x10, "10"),
                    (0x11, "11"),
                    (0x1f, "1f"),
                    (0xff, "ff"),
                    (0xFFFFFFFFFFFFFFFF, "ffffffffffffffff")
                ];

    for &(n, hex) in cases {
        println!("{}", &n);
        assert_eq!(u64_to_hex(n), String::from(hex))
    }

}
