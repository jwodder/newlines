use crate::iter::FindIndices;
use crate::nl::Newline;
use crate::nlset::NewlineSet;

mod private {
    #[allow(unnameable_types)]
    pub trait Sealed {}

    impl Sealed for super::Newline {}

    impl Sealed for super::NewlineSet {}
}

pub trait NewlinePattern: private::Sealed {
    fn search(&self, s: &str) -> Option<(usize, usize)>;
    fn rsearch(&self, s: &str) -> Option<(usize, usize)>;

    fn find_indices<'a>(&'a self, s: &'a str) -> FindIndices<'a, Self>
    where
        Self: Sized,
    {
        FindIndices::new(self, s)
    }
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

        mod find_indices {
            use super::*;

            mod ascii {
                use super::*;

                #[test]
                fn empty() {
                    let mut iter = NewlineSet::ASCII.find_indices("");
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next_back(), None);
                    assert_eq!(iter.next_back(), None);
                }

                #[test]
                fn no_newline() {
                    let mut iter = NewlineSet::ASCII.find_indices("foobar");
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next_back(), None);
                    assert_eq!(iter.next_back(), None);
                }

                #[rstest]
                #[case("\n", (0, 1))]
                #[case("\r", (0, 1))]
                #[case("\r\n", (0, 2))]
                #[case("foo\n", (3, 4))]
                #[case("foo\r", (3, 4))]
                #[case("foo\r\n", (3, 5))]
                #[case("\nfoo", (0, 1))]
                #[case("\rfoo", (0, 1))]
                #[case("\r\nfoo", (0, 2))]
                #[case("foo\nbar", (3, 4))]
                #[case("foo\rbar", (3, 4))]
                #[case("foo\r\nbar", (3, 5))]
                #[case("foo“\n”bar", (6, 7))]
                #[case("foo“\r”bar", (6, 7))]
                #[case("foo“\r\n”bar", (6, 8))]
                fn one_newline(#[case] s: &str, #[case] value: (usize, usize)) {
                    let mut iter = NewlineSet::ASCII.find_indices(s);
                    assert_eq!(iter.next(), Some(value));
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next_back(), None);
                    assert_eq!(iter.next_back(), None);
                    let mut riter = NewlineSet::ASCII.find_indices(s);
                    assert_eq!(riter.next_back(), Some(value));
                    assert_eq!(riter.next_back(), None);
                    assert_eq!(riter.next_back(), None);
                    assert_eq!(riter.next(), None);
                    assert_eq!(riter.next(), None);
                }

                #[rstest]
                #[case("\n\r", (0, 1), (1, 2))]
                #[case("foo\n\rbar", (3, 4), (4, 5))]
                #[case("foo\n\nbar", (3, 4), (4, 5))]
                #[case("foo\r\rbar", (3, 4), (4, 5))]
                #[case("foo\nbar\n", (3, 4), (7, 8))]
                #[case("foo\rbar\r", (3, 4), (7, 8))]
                #[case("foo\r\nbar\r\n", (3, 5), (8, 10))]
                fn two_newlines(
                    #[case] s: &str,
                    #[case] nel1: (usize, usize),
                    #[case] nel2: (usize, usize),
                ) {
                    let mut iter = NewlineSet::ASCII.find_indices(s);
                    assert_eq!(iter.next(), Some(nel1));
                    assert_eq!(iter.next(), Some(nel2));
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next(), None);
                    assert_eq!(iter.next_back(), None);
                    assert_eq!(iter.next_back(), None);
                    let mut riter = NewlineSet::ASCII.find_indices(s);
                    assert_eq!(riter.next_back(), Some(nel2));
                    assert_eq!(riter.next_back(), Some(nel1));
                    assert_eq!(riter.next_back(), None);
                    assert_eq!(riter.next_back(), None);
                    assert_eq!(riter.next(), None);
                    assert_eq!(riter.next(), None);
                    let mut diter = NewlineSet::ASCII.find_indices(s);
                    assert_eq!(diter.next(), Some(nel1));
                    assert_eq!(diter.next_back(), Some(nel2));
                    assert_eq!(diter.next(), None);
                    assert_eq!(diter.next(), None);
                    assert_eq!(diter.next_back(), None);
                    assert_eq!(diter.next_back(), None);
                }
            }
        }
    }

    // newline: find_indices()
    //  CR ~ \r\r\n
    //  CRLF ~ \r\r\n
    //  rev()
    //  next() mixed with next_back()

    // newline set: find_indices()
    //  {CR, CRLF} ~ \r\r\n
    //  rev()
    //  next() mixed with next_back()
}
