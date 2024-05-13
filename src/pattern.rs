use crate::nl::Newline;
use crate::nlset::NewlineSet;

mod private {
    #[allow(unnameable_types)]
    pub trait Sealed {}

    impl Sealed for super::Newline {}

    impl Sealed for super::NewlineSet {}
}

pub trait NewlinePattern: private::Sealed {
    // Panics if `start` is not on a character boundary
    fn search_after(&self, s: &str, start: usize) -> Option<(usize, usize)>;

    // Panics if `stop` is not on a character boundary
    fn rsearch_before(&self, s: &str, stop: usize) -> Option<(usize, usize)>;

    fn search(&self, s: &str) -> Option<(usize, usize)> {
        self.search_after(s, 0)
    }

    fn rsearch(&self, s: &str) -> Option<(usize, usize)> {
        self.rsearch_before(s, s.len())
    }
}

impl NewlinePattern for Newline {
    fn search_after(&self, s: &str, start: usize) -> Option<(usize, usize)> {
        let i = s[start..].find(self.as_str())?.saturating_add(start);
        let j = i.saturating_add(self.len_utf8());
        Some((i, j))
    }

    fn rsearch_before(&self, s: &str, stop: usize) -> Option<(usize, usize)> {
        let i = s[..stop].rfind(self.as_str())?;
        let j = i.saturating_add(self.len_utf8());
        Some((i, j))
    }
}

impl NewlinePattern for NewlineSet {
    fn search_after(&self, s: &str, start: usize) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        let i = s[start..].find(self.pattern())?.saturating_add(start);
        let length = if self.crlf && s[i..].starts_with("\r\n") {
            2
        } else {
            let Some(ch) = s[i..].chars().next() else {
                unreachable!(
                    "Nonempty NewlineSet pattern should have matched at start of a character"
                )
            };
            ch.len_utf8()
        };
        let j = i.saturating_add(length);
        Some((i, j))
    }

    fn rsearch_before(&self, s: &str, mut stop: usize) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        loop {
            let mut i = s[..stop].rfind(self.pattern())?;
            let length = match (self.crlf, self.pattern.contains('\n')) {
                (true, true) if s[i..stop].starts_with('\n') && s[..i].ends_with('\r') => {
                    i -= 1;
                    2
                }
                (true, false) if s[i..stop].starts_with("\r\n") => 2,
                _ => {
                    let Some(ch) = s[i..stop].chars().next() else {
                        unreachable!(
                            "Nonempty NewlineSet pattern should have matched at start of a character"
                        )
                    };
                    if !self.cr && ch == '\r' {
                        stop = i;
                        continue;
                    }
                    ch.len_utf8()
                }
            };
            let j = i.saturating_add(length);
            return Some((i, j));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    mod newline {
        use super::*;

        #[rstest]
        #[case(Newline::LineFeed, "foobar", None)]
        #[case(Newline::LineFeed, "foo\nbar", Some((3, 4)))]
        #[case(Newline::LineFeed, "\nfoobar", Some((0, 1)))]
        #[case(Newline::LineFeed, "foobar\n", Some((6, 7)))]
        #[case(Newline::LineFeed, "foo\rbar", None)]
        #[case(Newline::CrLf, "foo\rbar", None)]
        #[case(Newline::CrLf, "foo\nbar", None)]
        #[case(Newline::CarriageReturn, "foo\rbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn, "foo\r\nbar", Some((3, 4)))]
        #[case(Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::CrLf, "foo\rbar\r\nquux", Some((7, 9)))]
        #[case(Newline::LineSeparator, "foo\u{2028}bar", Some((3, 6)))]
        fn search(#[case] nl: Newline, #[case] s: &str, #[case] m: Option<(usize, usize)>) {
            assert_eq!(nl.search(s), m);
            if let Some((start, end)) = m {
                assert_eq!(&s[start..end], nl.as_str());
            }
        }

        #[rstest]
        #[case(Newline::LineFeed, "foobar", None)]
        #[case(Newline::LineFeed, "foo\nbar", Some((3, 4)))]
        #[case(Newline::LineFeed, "\nfoobar", Some((0, 1)))]
        #[case(Newline::LineFeed, "foobar\n", Some((6, 7)))]
        #[case(Newline::LineFeed, "foo\rbar", None)]
        #[case(Newline::CrLf, "foo\rbar", None)]
        #[case(Newline::CrLf, "foo\nbar", None)]
        #[case(Newline::CarriageReturn, "foo\rbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn, "foo\r\nbar", Some((3, 4)))]
        #[case(Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::CrLf, "foo\rbar\r\nquux", Some((7, 9)))]
        #[case(Newline::LineSeparator, "foo\u{2028}bar", Some((3, 6)))]
        fn rsearch(#[case] nl: Newline, #[case] s: &str, #[case] m: Option<(usize, usize)>) {
            assert_eq!(nl.rsearch(s), m);
            if let Some((start, end)) = m {
                assert_eq!(&s[start..end], nl.as_str());
            }
        }

        #[rstest]
        #[case(Newline::LineFeed, "foo\nbar", 7, None)]
        #[case(Newline::LineFeed, "foo\nbar", 4, None)]
        #[case(Newline::LineFeed, "foo\nbar", 3, Some((3, 4)))]
        #[case(Newline::LineFeed, "foo\nbar\nquux", 4, Some((7, 8)))]
        #[case(Newline::CrLf, "foo\r\nbar", 4, None)]
        fn search_after(
            #[case] nl: Newline,
            #[case] s: &str,
            #[case] start: usize,
            #[case] m: Option<(usize, usize)>,
        ) {
            assert_eq!(nl.search_after(s, start), m);
            if let Some((i, j)) = m {
                assert!(i >= start);
                assert_eq!(&s[i..j], nl.as_str());
            }
        }

        #[rstest]
        #[case(Newline::LineFeed, "foo\nbar", 0, None)]
        #[case(Newline::LineFeed, "foo\nbar", 3, None)]
        #[case(Newline::LineFeed, "foo\nbar", 4, Some((3, 4)))]
        #[case(Newline::CrLf, "foo\r\nbar", 4, None)]
        #[case(Newline::CrLf, "foo\r\nbar", 5, Some((3, 5)))]
        fn rsearch_before(
            #[case] nl: Newline,
            #[case] s: &str,
            #[case] stop: usize,
            #[case] m: Option<(usize, usize)>,
        ) {
            assert_eq!(nl.rsearch_before(s, stop), m);
            if let Some((i, j)) = m {
                assert!(j <= stop);
                assert_eq!(&s[i..j], nl.as_str());
            }
        }
    }

    mod newline_set {
        use super::*;

        #[rstest]
        #[case(NewlineSet::EMPTY, "foobar", None)]
        #[case(NewlineSet::EMPTY, "foo\r\nbar", None)]
        #[case(Newline::LineFeed.into(), "foobar", None)]
        #[case(Newline::LineFeed.into(), "foo\nbar", Some((3, 4)))]
        #[case(Newline::LineFeed.into(), "foo\r\nbar", Some((4, 5)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar\r\nquux", Some((3, 4)))]
        #[case(Newline::CarriageReturn.into(), "foo\r\nbar", Some((3, 4)))]
        #[case(Newline::LineSeparator.into(), "foo\u{2028}bar", Some((3, 6)))]
        #[case(Newline::LineFeed | Newline::CarriageReturn, "foo\rbar\nquux", Some((3, 4)))]
        #[case(Newline::LineFeed | Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        fn search(#[case] nlset: NewlineSet, #[case] s: &str, #[case] m: Option<(usize, usize)>) {
            assert_eq!(nlset.search(s), m);
            if let Some((start, end)) = m {
                assert!(nlset.contains(Newline::try_from(&s[start..end]).unwrap()));
            }
        }

        #[rstest]
        #[case(NewlineSet::ASCII, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::LineFeed | Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\r\nbar", Some((3, 5)))]
        #[case(Newline::CarriageReturn.into(), "foo\r\nbar", Some((3, 4)))]
        #[case(Newline::CrLf.into(), "foo\r\nbar", Some((3, 5)))]
        #[case(NewlineSet::ASCII, "foo\n\rbar", Some((4, 5)))]
        #[case(Newline::LineFeed | Newline::CrLf, "foo\n\rbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\n\rbar", Some((4, 5)))]
        #[case(Newline::CarriageReturn.into(), "foo\n\rbar", Some((4, 5)))]
        #[case(Newline::CrLf.into(), "foo\n\rbar", None)]
        #[case(Newline::LineFeed | Newline::CrLf, "foo\nbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\nbar", None)]
        #[case(Newline::CarriageReturn.into(), "foo\nbar", None)]
        #[case(Newline::CrLf.into(), "foo\nbar", None)]
        #[case(Newline::LineFeed | Newline::CrLf, "foo\rbar", None)]
        #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar", Some((3, 4)))]
        #[case(Newline::CarriageReturn.into(), "foo\rbar", Some((3, 4)))]
        #[case(Newline::CrLf.into(), "foo\rbar", None)]
        #[case(Newline::LineSeparator.into(), "foo\u{2028}bar", Some((3, 6)))]
        fn rsearch(#[case] nlset: NewlineSet, #[case] s: &str, #[case] m: Option<(usize, usize)>) {
            assert_eq!(nlset.rsearch(s), m);
            if let Some((start, end)) = m {
                assert!(nlset.contains(Newline::try_from(&s[start..end]).unwrap()));
            }
        }
    }

    // newline set: search_after
    // newline set: rsearch_before
    // - CRLF ~ \r<stop>\n
}
