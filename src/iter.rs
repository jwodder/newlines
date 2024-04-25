use super::charset::CharSetIter;
use super::nl::Newline;
use super::nlset::NewlineSet;
use std::iter::FusedIterator;

#[derive(Clone, Debug)]
pub struct IntoIter {
    inner: CharSetIter,
    cr: bool,
    crlf: bool,
}

impl IntoIter {
    pub(crate) fn new(nlset: NewlineSet) -> IntoIter {
        IntoIter {
            inner: nlset.pattern.into_iter(),
            cr: nlset.cr,
            crlf: nlset.crlf,
        }
    }
}

impl Iterator for IntoIter {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        match (self.inner.peek(), self.cr, self.crlf) {
            (Some('\r'), true, crlf) => {
                self.cr = false;
                if !crlf {
                    self.inner.next();
                }
                Some(Newline::CarriageReturn)
            }
            (Some('\r'), false, true) => {
                self.crlf = false;
                self.inner.next();
                Some(Newline::CrLf)
            }
            (Some(ch), _, _) => {
                let nl = Newline::try_from(ch).ok();
                debug_assert!(nl.is_some(), "Char in pattern buf should map to Newline");
                self.inner.next();
                nl
            }
            (None, _, _) => None,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut sz, _) = self.inner.size_hint();
        sz += usize::from(self.cr && self.crlf);
        (sz, Some(sz))
    }
}

impl FusedIterator for IntoIter {}

impl ExactSizeIterator for IntoIter {}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Newline> {
        match (self.inner.peek_back(), self.cr, self.crlf) {
            (Some('\r'), cr, true) => {
                self.crlf = false;
                if !cr {
                    self.inner.next_back();
                }
                Some(Newline::CrLf)
            }
            (Some('\r'), true, false) => {
                self.cr = false;
                self.inner.next_back();
                Some(Newline::CarriageReturn)
            }
            (Some(ch), _, _) => {
                let nl = Newline::try_from(ch).ok();
                debug_assert!(nl.is_some(), "Char in pattern buf should map to Newline");
                self.inner.next_back();
                nl
            }
            (None, _, _) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod into_iter {
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
}
