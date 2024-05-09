use super::inner::Char2Newline;
use super::AscendingNewlines;
use crate::charset::{CharSet, CharSetDiff, Diff};
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use std::iter::FusedIterator;

/// Iterator of all [`Newline`] variants in one [`NewlineSet`] but not another.
/// Values are unique and yielded in ascending order.
///
/// A `Difference` instance is acquired by calling
/// [`NewlineSet::difference()`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Difference(Char2Newline<InnerDifference>);

impl Difference {
    pub(crate) fn new(nlset1: NewlineSet, nlset2: NewlineSet) -> Difference {
        Difference(Char2Newline::new(
            InnerDifference::new(nlset1.pattern, nlset2.pattern),
            nlset1.cr && !nlset2.cr,
            nlset1.crlf && !nlset2.crlf,
        ))
    }
}

impl Iterator for Difference {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for Difference {}

impl ExactSizeIterator for Difference {}

impl AscendingNewlines for Difference {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct InnerDifference(CharSetDiff);

impl InnerDifference {
    pub(super) fn new(cs1: CharSet, cs2: CharSet) -> InnerDifference {
        InnerDifference(cs1.diff(cs2))
    }
}

impl Iterator for InnerDifference {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        for d in self.0.by_ref() {
            match d {
                Diff::Left(ch) => return Some(ch),
                Diff::Both('\r') => return Some('\r'),
                Diff::Both(_) | Diff::Right(_) => (),
            }
        }
        None
    }
}
