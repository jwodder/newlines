use super::inner::Char2Newline;
use super::AscendingNewlines;
use crate::charset::{CharSet, CharSetDiff, Diff};
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use core::iter::FusedIterator;

/// Iterator of all [`Newline`] variants in one but not both of a given pair of
/// [`NewlineSet`]s.  Values are unique and yielded in ascending order.
///
/// A `SymmetricDifference` instance is acquired by calling
/// [`NewlineSet::symmetric_difference()`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SymmetricDifference(Char2Newline<InnerSymmetricDifference>);

impl SymmetricDifference {
    pub(crate) fn new(nlset1: NewlineSet, nlset2: NewlineSet) -> SymmetricDifference {
        SymmetricDifference(Char2Newline::new(
            InnerSymmetricDifference::new(nlset1.pattern, nlset2.pattern),
            nlset1.cr ^ nlset2.cr,
            nlset1.crlf ^ nlset2.crlf,
        ))
    }
}

impl Iterator for SymmetricDifference {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for SymmetricDifference {}

impl ExactSizeIterator for SymmetricDifference {}

impl AscendingNewlines for SymmetricDifference {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerSymmetricDifference(CharSetDiff);

impl InnerSymmetricDifference {
    fn new(cs1: CharSet, cs2: CharSet) -> InnerSymmetricDifference {
        InnerSymmetricDifference(cs1.diff(cs2))
    }
}

impl Iterator for InnerSymmetricDifference {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        for d in self.0.by_ref() {
            match d {
                Diff::Left(ch) | Diff::Right(ch) => return Some(ch),
                Diff::Both('\r') => return Some('\r'),
                Diff::Both(_) => (),
            }
        }
        None
    }
}
