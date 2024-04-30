use super::inner::Char2Newline;
use super::AscendingNewlines;
use crate::charset::{CharSet, CharSetDiff, Diff};
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use std::iter::FusedIterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intersection(Char2Newline<InnerIntersection>);

impl Intersection {
    pub(crate) fn new(nlset1: NewlineSet, nlset2: NewlineSet) -> Intersection {
        Intersection(Char2Newline::new(
            InnerIntersection::new(nlset1.pattern, nlset2.pattern),
            nlset1.cr && nlset2.cr,
            nlset1.crlf && nlset2.crlf,
        ))
    }
}

impl Iterator for Intersection {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for Intersection {}

impl ExactSizeIterator for Intersection {}

impl AscendingNewlines for Intersection {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerIntersection(CharSetDiff);

impl InnerIntersection {
    fn new(cs1: CharSet, cs2: CharSet) -> InnerIntersection {
        InnerIntersection(cs1.diff(cs2))
    }
}

impl Iterator for InnerIntersection {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        for d in self.0.by_ref() {
            if let Diff::Both(ch) = d {
                return Some(ch);
            }
        }
        None
    }
}
