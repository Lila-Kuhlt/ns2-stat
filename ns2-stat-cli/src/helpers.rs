use std::cell::RefCell;
use std::fmt;

pub struct FormatWith<'a, I, F> {
    iter: RefCell<Option<(I, F)>>,
    separator: &'a str,
}

pub fn format_with<I, S, F>(iter: I, separator: &str, f: F) -> FormatWith<I, F>
where
    I: Iterator<Item = S>,
    F: FnMut(&mut fmt::Formatter<'_>, S) -> fmt::Result,
{
    FormatWith {
        iter: RefCell::new(Some((iter, f))),
        separator,
    }
}

impl<I, S, F> fmt::Display for FormatWith<'_, I, F>
where
    I: Iterator<Item = S>,
    F: FnMut(&mut fmt::Formatter<'_>, S) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (mut iter, mut callback) = self.iter.borrow_mut().take().expect("Join was already formatted");
        if let Some(x) = iter.next() {
            callback(f, x)?;
            for x in iter {
                write!(f, "{}", self.separator)?;
                callback(f, x)?;
            }
        }
        Ok(())
    }
}
