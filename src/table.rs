#[macro_export]
macro_rules! row {
    ($($e:literal),*) => {
        [$(format!($e)),*]
    }
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

pub fn print_table<T, const N: usize>(titles: [&str; N], alignments: [Alignment; N], table: &[T], formatter: impl Fn(&T) -> [String; N]) {
    let mut lengths = [0; N]; // `lengths[i]` is the length of the ith column
    let rows = table.iter().map(formatter).collect::<Vec<_>>();
    for i in 0..N {
        lengths[i] = std::cmp::max(
            titles[i].len(),
            rows.iter().map(|row| row[i].len()).max().unwrap_or(0)
        );
    }

    for i in 0..N {
        print!("{:width$}    ", titles[i], width = lengths[i]);
    }
    println!();
    for row in rows {
        for i in 0..N {
            let content = &row[i];
            let alignment = alignments[i];
            let len = lengths[i];
            match alignment {
                Alignment::Left => print!("{:<width$}    ", content, width = len),
                Alignment::Center => print!("{:^width$}    ", content, width = len),
                Alignment::Right => print!("{:>width$}    ", content, width = len),
            }
        }
        println!();
    }
}
