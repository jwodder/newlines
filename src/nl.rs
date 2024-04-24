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
}

impl fmt::Display for Newline {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
