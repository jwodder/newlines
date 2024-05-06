use super::diff::InnerDifference;
use super::inner::Char2Newline;
use super::AscendingNewlines;
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use std::iter::FusedIterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Complement(Char2Newline<InnerDifference>);

impl Complement {
    pub(crate) fn new(nlset: NewlineSet) -> Complement {
        Complement(Char2Newline::new(
            InnerDifference::new(NewlineSet::ALL.pattern, nlset.pattern),
            !nlset.cr,
            !nlset.crlf,
        ))
    }
}

impl Iterator for Complement {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for Complement {}

impl ExactSizeIterator for Complement {}

impl AscendingNewlines for Complement {}
