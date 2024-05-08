use crate::nl::Newline;
use crate::nlset::NewlineSet;

mod private {
    pub trait Sealed {}

    impl Sealed for super::Newline {}

    impl Sealed for super::NewlineSet {}
}

pub trait NewlinePattern: private::Sealed {
    fn search(&self, s: &str) -> Option<(usize, usize)>;
    fn rsearch(&self, s: &str) -> Option<(usize, usize)>;
}

impl NewlinePattern for Newline {
    fn search(&self, s: &str) -> Option<(usize, usize)> {
        let start = s.find(self.as_str())?;
        let end = start.saturating_add(self.len_utf8());
        Some((start, end))
    }

    fn rsearch(&self, s: &str) -> Option<(usize, usize)> {
        let start = s.rfind(self.as_str())?;
        let end = start.saturating_add(self.len_utf8());
        Some((start, end))
    }
}

impl NewlinePattern for NewlineSet {
    fn search(&self, s: &str) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        let start = s.find(self.pattern())?;
        let length = if self.crlf && s[start..].starts_with("\r\n") {
            2
        } else {
            let Some(ch) = s[start..].chars().next() else {
                unreachable!(
                    "Nonempty NewlineSet pattern should have matched at start of a character"
                )
            };
            ch.len_utf8()
        };
        let end = start.saturating_add(length);
        Some((start, end))
    }

    fn rsearch(&self, s: &str) -> Option<(usize, usize)> {
        if self.is_empty() {
            return None;
        }
        let mut s_end = s.len();
        loop {
            let mut start = s[..s_end].rfind(self.pattern())?;
            let length = match (self.crlf, self.pattern.contains('\n')) {
                (true, true) if s[start..].starts_with('\n') && s[..start].ends_with('\r') => {
                    start -= 1;
                    2
                }
                (true, false) if s[start..].starts_with("\r\n") => 2,
                _ => {
                    let Some(ch) = s[start..].chars().next() else {
                        unreachable!(
                        "Nonempty NewlineSet pattern should have matched at start of a character"
                    )
                    };
                    if !self.cr && ch == '\r' {
                        s_end = start;
                        continue;
                    }
                    ch.len_utf8()
                }
            };
            let end = start.saturating_add(length);
            return Some((start, end));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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
    fn test_newline_search(
        #[case] nl: Newline,
        #[case] s: &str,
        #[case] m: Option<(usize, usize)>,
    ) {
        assert_eq!(nl.search(s), m);
        if let Some((start, end)) = m {
            assert_eq!(&s[start..end], nl.as_str());
        }
    }

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
    fn test_newline_set_search(
        #[case] nlset: NewlineSet,
        #[case] s: &str,
        #[case] m: Option<(usize, usize)>,
    ) {
        assert_eq!(nlset.search(s), m);
        if let Some((start, end)) = m {
            assert!(nlset.contains(Newline::try_from(&s[start..end]).unwrap()));
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
    fn test_newline_rsearch(
        #[case] nl: Newline,
        #[case] s: &str,
        #[case] m: Option<(usize, usize)>,
    ) {
        assert_eq!(nl.rsearch(s), m);
        if let Some((start, end)) = m {
            assert_eq!(&s[start..end], nl.as_str());
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
    fn test_newline_set_rsearch(
        #[case] nlset: NewlineSet,
        #[case] s: &str,
        #[case] m: Option<(usize, usize)>,
    ) {
        assert_eq!(nlset.rsearch(s), m);
        if let Some((start, end)) = m {
            assert!(nlset.contains(Newline::try_from(&s[start..end]).unwrap()));
        }
    }
}
