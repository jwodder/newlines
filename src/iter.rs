use super::nl::Newline;
use super::nlset::{NewlineSet, PatternBuf};
use std::iter::FusedIterator;

#[derive(Clone, Debug)]
pub struct IntoIter {
    pattern_buf: PatternBuf,
    pattern_len: usize,
    i: usize,
    cr: bool,
    crlf: bool,
}

impl IntoIter {
    pub(crate) fn new(nlset: NewlineSet) -> IntoIter {
        IntoIter {
            pattern_buf: nlset.pattern_buf,
            pattern_len: nlset.pattern_len,
            i: 0,
            cr: nlset.cr,
            crlf: nlset.crlf,
        }
    }
}

impl Iterator for IntoIter {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        if self.i >= self.pattern_len {
            return None;
        }
        match (self.pattern_buf[self.i], self.cr, self.crlf) {
            ('\r', true, crlf) => {
                self.cr = false;
                if !crlf {
                    self.i += 1;
                }
                Some(Newline::CarriageReturn)
            }
            ('\r', false, true) => {
                self.crlf = false;
                self.i += 1;
                Some(Newline::CrLf)
            }
            (ch, _, _) => {
                let nl = Newline::try_from(ch).ok();
                debug_assert!(nl.is_some(), "Char in pattern buf should map to Newline");
                self.i += 1;
                nl
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.pattern_len - self.i + usize::from(self.cr && self.crlf);
        (sz, Some(sz))
    }
}

impl FusedIterator for IntoIter {}

impl ExactSizeIterator for IntoIter {}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Newline> {
        if self.i >= self.pattern_len {
            return None;
        }
        match (self.pattern_buf[self.pattern_len - 1], self.cr, self.crlf) {
            ('\r', cr, true) => {
                self.crlf = false;
                if !cr {
                    self.pattern_len -= 1;
                }
                Some(Newline::CrLf)
            }
            ('\r', true, false) => {
                self.cr = false;
                self.pattern_len -= 1;
                Some(Newline::CarriageReturn)
            }
            (ch, _, _) => {
                let nl = Newline::try_from(ch).ok();
                debug_assert!(nl.is_some(), "Char in pattern buf should map to Newline");
                self.pattern_len -= 1;
                nl
            }
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
