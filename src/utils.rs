
pub fn repeat_str<T: AsRef<str>>(s: T, times: usize) -> String {
    s.as_ref().chars().cycle().take(times).collect()
}


pub fn erase_up(lines: usize) {
    for _ in 0..lines {
        print!("\x1B[2K\x1B[A");
    }
    print!("\r");
}




// struct RecursiveIterator<U: Into<Option<T>>, T: Iterator<Item=U>>(T);


// struct BFSIterator<U: Into<Option<T>>, T: Iterator<Item=U>> {
//     iterator: RecursiveIterator<U, T>
// }
// impl<U: Into<Option<T>>, T: Iterator<Item=U>> Iterator for BFSIterator<U, T> {
//     type Item = U;

//     fn next(&mut self) -> Option<Self::Item> {
//         for x in self.iterator.0 {

//         };
//         None
//     }
// }
