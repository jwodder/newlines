use super::inner::Char2Newline;
use super::AscendingNewlines;
use crate::charset::{CharSet, CharSetDiff, Diff};
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use std::iter::FusedIterator;

/// Iterator of all [`Newline`] variants in one or both of a given pair of
/// [`NewlineSet`]s.  Values are unique and yielded in ascending order.
///
/// A `Union` instance is acquired by calling [`NewlineSet::union()`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Union(Char2Newline<InnerUnion>);

impl Union {
    pub(crate) fn new(nlset1: NewlineSet, nlset2: NewlineSet) -> Union {
        Union(Char2Newline::new(
            InnerUnion::new(nlset1.pattern, nlset2.pattern),
            nlset1.cr || nlset2.cr,
            nlset1.crlf || nlset2.crlf,
        ))
    }
}

impl Iterator for Union {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for Union {}

impl ExactSizeIterator for Union {}

impl AscendingNewlines for Union {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerUnion(CharSetDiff);

impl InnerUnion {
    fn new(cs1: CharSet, cs2: CharSet) -> InnerUnion {
        InnerUnion(cs1.diff(cs2))
    }
}

impl Iterator for InnerUnion {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self.0.next()? {
            Diff::Left(ch) | Diff::Both(ch) | Diff::Right(ch) => Some(ch),
        }
    }
}
