use super::inner::Char2Newline;
use crate::charset::CharSetIter;
use crate::nl::Newline;
use crate::nlset::NewlineSet;
use std::iter::FusedIterator;

#[derive(Clone, Debug)]
pub struct IntoIter(Char2Newline<CharSetIter>);

impl IntoIter {
    pub(crate) fn new(nlset: NewlineSet) -> IntoIter {
        IntoIter(Char2Newline::new(
            nlset.pattern.into_iter(),
            nlset.cr,
            nlset.crlf,
        ))
    }
}

impl Iterator for IntoIter {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl FusedIterator for IntoIter {}

impl ExactSizeIterator for IntoIter {}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Newline> {
        self.0.next_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let mut iter = NewlineSet::new().into_iter();
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn empty_rev() {
        let mut iter = NewlineSet::new().into_iter().rev();
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn singleton() {
        let mut iter = NewlineSet::from(Newline::FormFeed).into_iter();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::FormFeed));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn singleton_rev() {
        let mut iter = NewlineSet::from(Newline::FormFeed).into_iter().rev();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::FormFeed));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cr() {
        let mut iter = NewlineSet::from(Newline::CarriageReturn).into_iter();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cr_rev() {
        let mut iter = NewlineSet::from(Newline::CarriageReturn).into_iter().rev();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn crlf() {
        let mut iter = NewlineSet::from(Newline::CrLf).into_iter();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn crlf_rev() {
        let mut iter = NewlineSet::from(Newline::CrLf).into_iter().rev();
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cr_crlf() {
        let mut iter = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf]).into_iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn cr_crlf_rev() {
        let mut iter = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf])
            .into_iter()
            .rev();
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn all() {
        let mut iter = NewlineSet::from_iter(Newline::iter()).into_iter();
        assert_eq!(iter.size_hint(), (8, Some(8)));
        assert_eq!(iter.next(), Some(Newline::LineFeed));
        assert_eq!(iter.size_hint(), (7, Some(7)));
        assert_eq!(iter.next(), Some(Newline::VerticalTab));
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some(Newline::FormFeed));
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(Newline::NextLine));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::LineSeparator));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::ParagraphSeparator));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn all_rev() {
        let mut iter = NewlineSet::from_iter(Newline::iter()).into_iter().rev();
        assert_eq!(iter.size_hint(), (8, Some(8)));
        assert_eq!(iter.next(), Some(Newline::ParagraphSeparator));
        assert_eq!(iter.size_hint(), (7, Some(7)));
        assert_eq!(iter.next(), Some(Newline::LineSeparator));
        assert_eq!(iter.size_hint(), (6, Some(6)));
        assert_eq!(iter.next(), Some(Newline::NextLine));
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(Newline::FormFeed));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::VerticalTab));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::LineFeed));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }
}
