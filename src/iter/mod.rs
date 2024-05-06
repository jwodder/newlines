mod complement;
mod diff;
mod inner;
mod intersection;
mod into_iter;
mod symdiff;
mod union;
pub use self::complement::*;
pub use self::diff::*;
pub use self::intersection::*;
pub use self::into_iter::*;
pub use self::symdiff::*;
pub use self::union::*;
use crate::nl::{CharType, Newline};
use crate::nlset::NewlineSet;

pub(crate) trait AscendingNewlines: Iterator<Item = Newline> + Sized {
    fn into_newline_set(self) -> NewlineSet {
        let mut nlset = NewlineSet::new();
        let mut prev_char = None;
        for nl in self {
            let ch = match nl.chartype() {
                CharType::Char('\r') => {
                    nlset.cr = true;
                    '\r'
                }
                CharType::Char(ch) => ch,
                CharType::CrLf => {
                    nlset.crlf = true;
                    '\r'
                }
            };
            if prev_char.replace(ch) != Some(ch) {
                nlset.pattern.append(ch);
            }
        }
        nlset
    }
}
