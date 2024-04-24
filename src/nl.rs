use super::errors::{TryFromCharError, TryFromStrError};
use std::fmt;
use strum::{EnumCount, EnumIter};

#[derive(Copy, Clone, Debug, EnumCount, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Newline {
    LineFeed,
    CarriageReturn,
    CrLf,
    VerticalTab,
    FormFeed,
    NextLine,           // 0x85
    LineSeparator,      // U+2028
    ParagraphSeparator, // U+2029
}

impl Newline {
    // To avoid the need for users to import the trait
    pub const COUNT: usize = <Newline as EnumCount>::COUNT;

    /// Returns an iterator over all [`Newline`] variants
    pub fn iter() -> NewlineIter {
        // To avoid the need for users to import the trait
        <Newline as strum::IntoEnumIterator>::iter()
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Newline::LineFeed => "\n",
            Newline::CarriageReturn => "\r",
            Newline::CrLf => "\r\n",
            Newline::VerticalTab => "\x0B",
            Newline::FormFeed => "\x0C",
            Newline::NextLine => "\u{0085}",
            Newline::LineSeparator => "\u{2028}",
            Newline::ParagraphSeparator => "\u{2029}",
        }
    }

    pub fn as_char(&self) -> Option<char> {
        match self {
            Newline::LineFeed => Some('\n'),
            Newline::CarriageReturn => Some('\r'),
            Newline::CrLf => None,
            Newline::VerticalTab => Some('\x0B'),
            Newline::FormFeed => Some('\x0C'),
            Newline::NextLine => Some('\u{0085}'),
            Newline::LineSeparator => Some('\u{2028}'),
            Newline::ParagraphSeparator => Some('\u{2029}'),
        }
    }

    pub fn len_char(&self) -> usize {
        match self {
            Newline::CrLf => 2,
            _ => 1,
        }
    }

    pub fn len_utf8(&self) -> usize {
        self.as_str().len()
    }
}

impl fmt::Display for Newline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<char> for Newline {
    type Error = TryFromCharError;

    fn try_from(value: char) -> Result<Newline, TryFromCharError> {
        match value {
            '\n' => Ok(Newline::LineFeed),
            '\r' => Ok(Newline::CarriageReturn),
            '\x0B' => Ok(Newline::VerticalTab),
            '\x0C' => Ok(Newline::FormFeed),
            '\u{0085}' => Ok(Newline::NextLine),
            '\u{2028}' => Ok(Newline::LineSeparator),
            '\u{2029}' => Ok(Newline::ParagraphSeparator),
            ch => Err(TryFromCharError(ch)),
        }
    }
}

impl TryFrom<&str> for Newline {
    type Error = TryFromStrError;

    fn try_from(value: &str) -> Result<Newline, TryFromStrError> {
        match value {
            "\n" => Ok(Newline::LineFeed),
            "\r" => Ok(Newline::CarriageReturn),
            "\r\n" => Ok(Newline::CrLf),
            "\x0B" => Ok(Newline::VerticalTab),
            "\x0C" => Ok(Newline::FormFeed),
            "\u{0085}" => Ok(Newline::NextLine),
            "\u{2028}" => Ok(Newline::LineSeparator),
            "\u{2029}" => Ok(Newline::ParagraphSeparator),
            _ => Err(TryFromStrError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len_char() {
        for nl in Newline::iter() {
            assert_eq!(nl.len_char(), nl.as_str().chars().count());
        }
    }

    #[test]
    fn test_as_char() {
        for nl in Newline::iter() {
            let mut chiter = nl.as_str().chars();
            let c1 = chiter.next().unwrap();
            if chiter.next().is_some() {
                assert_eq!(nl.as_char(), None);
            } else {
                assert_eq!(nl.as_char(), Some(c1));
            }
        }
    }

    #[test]
    fn test_try_from_char() {
        for nl in Newline::iter() {
            if let Some(ch) = nl.as_char() {
                assert_eq!(Newline::try_from(ch), Ok(nl));
            }
        }
    }

    #[test]
    fn test_try_from_str() {
        for nl in Newline::iter() {
            assert_eq!(Newline::try_from(nl.as_str()), Ok(nl));
        }
    }
}
