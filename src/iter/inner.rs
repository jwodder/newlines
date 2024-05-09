use crate::nl::Newline;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Char2Newline<I> {
    inner: I,
    cr: bool,
    crlf: bool,
    queued: Option<Newline>,
    queued_back: Option<Newline>,
}

impl<I> Char2Newline<I> {
    pub(crate) fn new(inner: I, cr: bool, crlf: bool) -> Self {
        Char2Newline {
            inner,
            cr,
            crlf,
            queued: None,
            queued_back: None,
        }
    }
}

impl<I: Iterator<Item = char>> Iterator for Char2Newline<I> {
    type Item = Newline;

    fn next(&mut self) -> Option<Newline> {
        if let Some(nl) = self.queued.take() {
            return Some(nl);
        }
        loop {
            let Some(ch) = self.inner.next() else {
                return core::mem::take(&mut self.queued_back);
            };
            match (ch, self.cr, self.crlf) {
                ('\r', true, crlf) => {
                    if crlf {
                        self.queued = Some(Newline::CrLf);
                        // So that size_hint() won't add 1:
                        self.cr = false;
                    }
                    return Some(Newline::CarriageReturn);
                }
                ('\r', false, true) => return Some(Newline::CrLf),
                // ↓ This can happen when, for example, computing
                // ↓ `Intersection` for {CR} and {CRLF}
                ('\r', false, false) => (), // Go to next element of inner iter
                (ch, _, _) => {
                    let nl = Newline::try_from(ch).ok();
                    debug_assert!(
                        nl.is_some(),
                        "Char from inner iterator should map to Newline"
                    );
                    return nl;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.inner.size_hint();
        let mut inc = 0;
        if self.cr && self.crlf {
            inc += 1;
        }
        if self.queued.is_some() {
            inc += 1;
        }
        if self.queued_back.is_some() {
            inc += 1;
        }
        (lower + inc, upper.map(|i| i + inc))
    }
}

impl<I: DoubleEndedIterator<Item = char>> DoubleEndedIterator for Char2Newline<I> {
    fn next_back(&mut self) -> Option<Newline> {
        if let Some(nl) = self.queued_back.take() {
            return Some(nl);
        }
        let Some(ch) = self.inner.next_back() else {
            return core::mem::take(&mut self.queued);
        };
        match (ch, self.cr, self.crlf) {
            ('\r', cr, true) => {
                if cr {
                    self.queued_back = Some(Newline::CarriageReturn);
                    // So that size_hint() won't add 1:
                    self.crlf = false;
                }
                Some(Newline::CrLf)
            }
            ('\r', true, false) => Some(Newline::CarriageReturn),
            (ch, _, _) => {
                let nl = Newline::try_from(ch).ok();
                debug_assert!(
                    nl.is_some(),
                    "Char from inner iterator should map to Newline"
                );
                nl
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cr() {
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, false);
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, false);
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next_back(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn crlf() {
        let mut iter = Char2Newline::new(core::iter::once('\r'), false, true);
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
        let mut iter = Char2Newline::new(core::iter::once('\r'), false, true);
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn cr_crlf() {
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, true);
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, true);
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next_back(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, true);
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        let mut iter = Char2Newline::new(core::iter::once('\r'), true, true);
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn around_cr_crlf() {
        let mut iter = Char2Newline::new(['\n', '\r', '\u{0085}'].into_iter(), true, true);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next(), Some(Newline::LineFeed));
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next_back(), Some(Newline::NextLine));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
        let mut iter = Char2Newline::new(['\n', '\r', '\u{0085}'].into_iter(), true, true);
        assert_eq!(iter.size_hint(), (4, Some(4)));
        assert_eq!(iter.next_back(), Some(Newline::NextLine));
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(Newline::LineFeed));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next_back(), Some(Newline::CrLf));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(Newline::CarriageReturn));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}
