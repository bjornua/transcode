use std::cmp;
use std::borrow::Cow;
use std::iter::repeat;
use std::iter::once;
use utils;

#[derive(Clone, Debug)]
pub enum Alignment<'a> {
    Left(Cow<'a, str>),
    Right(Cow<'a, str>),
    Center(Cow<'a, str>),
    Empty,
}

pub enum Cell<'a> {
    Text(Alignment<'a>),
    Float(Cow<'a, f64>, usize),
    Integer(Cow<'a, i64>),
    Empty,
}

use self::Cell::{Text, Float, Integer};
use self::Alignment::{Left, Right, Center};

impl<'a> Cell<'a> {
    fn to_string(self) -> Alignment<'a> {
        match self {
            Text(v) => v,
            Float(v, prec) => Right(Cow::Owned(format!("{:.*}", prec, v))),
            Integer(v) => Right(Cow::Owned(format!("{}", v))),
            Cell::Empty => Alignment::Empty,
        }
    }
}
impl<'a> Alignment<'a> {
    fn len(&self) -> usize {
        match *self {
            Alignment::Empty => 0,
            Left(ref t) => t.chars().count(),
            Right(ref t) => t.chars().count(),
            Center(ref t) => t.chars().count(),
        }
    }

    fn to_padded_string(self, width: usize) -> Cow<'a, str> {
        if self.len() > width {
            return match self {
                Alignment::Empty => Cow::Borrowed(""),
                Left(t) | Right(t) | Center(t) => Cow::Owned(t.split_at(width).0.to_string()),
            };
        }

        let space = width - self.len();

        let (padding_left, padding_right, text) = match self {
            Alignment::Empty => (0, space, Cow::Borrowed("")),
            Left(t) => (0, space, t),
            Right(t) => (space, 0, t),
            Center(t) => {
                let half = space / 2;
                (half, space - half, t)
            }
        };

        Cow::Owned([Cow::Owned(utils::repeat_str(" ", padding_left)),
                    text,
                    Cow::Owned(utils::repeat_str(" ", padding_right))]
            .concat())
    }
}


fn max_slice<A: Clone + Ord, C: AsRef<[A]>>(a: C, b: C) -> Vec<A> {
    let a = a.as_ref();
    let b = b.as_ref();

    let (a, b) = if a.len() < b.len() { (b, a) } else { (a, b) };

    let b = b.iter().map(|x| Some(x)).chain(repeat(None));

    a.iter()
        .zip(b)
        .map(|(a, b)| match b {
            Some(b) => cmp::max(a, b).clone(),
            None => a.clone(),
        })
        .collect()
}


type Row<'a> = Vec<Cell<'a>>;
pub trait Table<'a>: Iterator<Item = Row<'a>> {}
impl<'a, T: Iterator<Item = Row<'a>>> Table<'a> for T {}

fn expand_row<'a>(row: Vec<Alignment<'a>>, num_columns: usize) -> Vec<Alignment<'a>> {
    use std::iter::repeat;
    if row.len() < num_columns {
        let missing = num_columns - row.len();
        row.into_iter().chain(repeat(Alignment::Empty).take(missing)).collect()
    } else {
        row
    }
}

fn make_table<'a, T: Table<'a>>(rows: T) -> Vec<Vec<Cow<'a, str>>> {
    let rows: Vec<Vec<_>> = rows.map(|row| row.into_iter().map(|col| col.to_string()).collect())
        .collect();

    let num_columns = match (&rows).into_iter().map(|row| row.len()).max() {
        Some(n) => n,
        None => return Vec::new(),
    };

    let columns_width = (&rows).into_iter().fold(Vec::<_>::new(), |acc, row| {
        max_slice(acc,
                  row.iter()
                      .map(|col| col.len())
                      .collect())
    });

    let rows_expanded = rows.into_iter().map(|row| expand_row(row, num_columns));

    rows_expanded.map(|row| {
            row.into_iter()
                .zip(&columns_width)
                .map(|(col, &width)| col.to_padded_string(width))
                .collect()
        })
        .collect()
}

pub fn print_table<'a, T: Table<'a>>(headers: Option<Vec<&'a str>>, rows: T) -> usize {
    let rows = match headers {
        None => make_table(rows),
        Some(s) => {
            let headers = s.into_iter();
            let headers: Vec<_> = headers.map(|c| Text(Left(Cow::Borrowed(c)))).collect();
            make_table(once(headers).chain(rows))
        }
    };
    let lines = rows.len();
    for row in rows {
        print!("{}", row.join(" | "));
        print!("\n");
    }

    use std::io::{stderr, Write};
    let _ = stderr().flush();
    lines
}
