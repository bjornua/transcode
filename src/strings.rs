
use std::borrow::Cow;

pub fn truncate_middle<'a>(a: Cow<'a, str>,
                           filler: &'static str,
                           max_width: usize)
                           -> Cow<'a, str> {
    let length = a.len();

    if length <= max_width {
        return a;
    }

    let filler_length = filler.len();

    if max_width < filler_length + 2 {
        match a {
            Cow::Borrowed(a) => a[..max_width].into(),
            Cow::Owned(a) => a[..max_width].to_string().into(),
        }
    } else {
        let cut = length - max_width + filler_length;
        let middle = length / 2;

        let cut_left = cut / 2;
        let cut_right = cut - cut_left;
        vec![&a[..(middle - cut_left)], filler, &a[(middle + cut_right)..]].concat().into()
    }
}

pub fn truncate_left<'a>(a: Cow<'a, str>, filler: &'static str, max_width: usize) -> Cow<'a, str> {
    let length = a.len();

    if length <= max_width {
        return a;
    }

    let filler_length = filler.len();

    if max_width < filler_length + 1 {
        match a {
            Cow::Borrowed(a) => a[..max_width].into(),
            Cow::Owned(a) => a[..max_width].to_string().into(),
        }
    } else {
        let cut = length - max_width + filler_length;
        vec![filler, &a[cut..]].concat().into()
    }
}
