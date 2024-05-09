use super::errors::{TryFromCharError, TryFromStrError};
use std::fmt;
use strum::{EnumCount, EnumIter};

/// An enumeration of various newline sequences.  This includes both typical
/// newlines used on major operating systems and characters that the [Unicode
/// Line Breaking Algorithm][tr14] treats as mandatory line breaks.
///
/// Note that the order of the variants matches the lexicographic order of the
/// variants' newline sequences as strings.
///
/// [tr14]: https://www.unicode.org/reports/tr14/
#[derive(Copy, Clone, Debug, EnumCount, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Newline {
    /// U+000A LINE FEED (LF), the newline sequence used on Unix-like systems
    ///
    /// Representable as `'\n'` in various programming languages
    LineFeed,

    /// U+000B LINE TABULATION (a.k.a. "vertical tab," "VTAB," or "VT")
    ///
    /// Representable as `'\v'` in various programming languages (not Rust)
    VerticalTab,

    /// U+000C FORM FEED (FF), often used to separate pages of text
    ///
    /// Representable as `'\f'` in various programming languages (not Rust)
    FormFeed,

    /// U+000D CARRIAGE RETURN (CR), the newline sequence used on Mac OS 9.x
    /// and earlier
    ///
    /// Representable as `'\r'` in various programming languages
    CarriageReturn,

    /// <U+000A, U+000D>, a carriage return character followed by a line feed
    /// character.  This is the newline sequence used on Windows and by many
    /// text-based internet protocols.
    ///
    /// This is the only multi-character newline sequence recognized by this
    /// library.
    CrLf,

    /// U+0085 NEXT LINE (NEL), the Unicode equivalent of the newline sequence
    /// used on EBCDIC-based systems
    NextLine,

    /// U+2028 LINE SEPARATOR
    LineSeparator,

    /// U+2029 PARAGRAPH SEPARATOR
    ParagraphSeparator,
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
            Newline::VerticalTab => "\x0B",
            Newline::FormFeed => "\x0C",
            Newline::CarriageReturn => "\r",
            Newline::CrLf => "\r\n",
            Newline::NextLine => "\u{0085}",
            Newline::LineSeparator => "\u{2028}",
            Newline::ParagraphSeparator => "\u{2029}",
        }
    }

    pub fn as_char(&self) -> Option<char> {
        match self.chartype() {
            CharType::Char(ch) => Some(ch),
            CharType::CrLf => None,
        }
    }

    pub(crate) fn chartype(&self) -> CharType {
        match self {
            Newline::LineFeed => CharType::Char('\n'),
            Newline::VerticalTab => CharType::Char('\x0B'),
            Newline::FormFeed => CharType::Char('\x0C'),
            Newline::CarriageReturn => CharType::Char('\r'),
            Newline::CrLf => CharType::CrLf,
            Newline::NextLine => CharType::Char('\u{0085}'),
            Newline::LineSeparator => CharType::Char('\u{2028}'),
            Newline::ParagraphSeparator => CharType::Char('\u{2029}'),
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
            '\x0B' => Ok(Newline::VerticalTab),
            '\x0C' => Ok(Newline::FormFeed),
            '\r' => Ok(Newline::CarriageReturn),
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
            "\x0B" => Ok(Newline::VerticalTab),
            "\x0C" => Ok(Newline::FormFeed),
            "\r" => Ok(Newline::CarriageReturn),
            "\r\n" => Ok(Newline::CrLf),
            "\u{0085}" => Ok(Newline::NextLine),
            "\u{2028}" => Ok(Newline::LineSeparator),
            "\u{2029}" => Ok(Newline::ParagraphSeparator),
            _ => Err(TryFromStrError),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum CharType {
    Char(char),
    CrLf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[test]
    fn variants_sorted_by_str() {
        for (before, after) in Newline::iter().tuple_windows() {
            let s1 = before.as_str();
            let s2 = after.as_str();
            assert!(s1 < s2, "{s1:?} >= {s2:?}");
        }
    }

    #[test]
    fn only_crlf_is_multichar() {
        for nl in Newline::iter() {
            if nl == Newline::CrLf {
                assert!(nl.len_char() > 1);
                assert!(nl.as_char().is_none());
            } else {
                assert!(nl.len_char() == 1);
                assert!(nl.as_char().is_some());
            }
        }
    }

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
