use crate::nl::Newline;
use crate::nlset::NewlineSet;

mod private {
    pub trait Sealed {}

    impl Sealed for super::Newline {}

    impl Sealed for super::NewlineSet {}
}

pub trait NewlinePattern: private::Sealed {
    fn search<'a>(&self, s: &'a str) -> Option<Match<'a>>;
    fn rsearch<'a>(&self, s: &'a str) -> Option<Match<'a>>;
}

impl NewlinePattern for Newline {
    fn search<'a>(&self, s: &'a str) -> Option<Match<'a>> {
        let start = s.find(self.as_str())?;
        let end = start.saturating_add(self.len_utf8());
        Some(Match {
            start,
            end,
            newline: *self,
            before: &s[..start],
            after: &s[end..],
        })
    }

    fn rsearch<'a>(&self, s: &'a str) -> Option<Match<'a>> {
        let start = s.rfind(self.as_str())?;
        let end = start.saturating_add(self.len_utf8());
        Some(Match {
            start,
            end,
            newline: *self,
            before: &s[..start],
            after: &s[end..],
        })
    }
}

impl NewlinePattern for NewlineSet {
    fn search<'a>(&self, s: &'a str) -> Option<Match<'a>> {
        if self.is_empty() {
            return None;
        }
        let start = s.find(self.pattern())?;
        let newline = if self.crlf && s[start..].starts_with("\r\n") {
            Newline::CrLf
        } else {
            let Some(ch) = s[start..].chars().next() else {
                unreachable!(
                    "Nonempty NewlineSet pattern should have matched at start of a character"
                )
            };
            match Newline::try_from(ch) {
                Ok(nl) => nl,
                Err(_) => unreachable!(
                    "NewlineSet pattern should have matched a char that maps to Newline"
                ),
            }
        };
        let end = start.saturating_add(newline.len_utf8());
        Some(Match {
            start,
            end,
            newline,
            before: &s[..start],
            after: &s[end..],
        })
    }

    fn rsearch<'a>(&self, s: &'a str) -> Option<Match<'a>> {
        if self.is_empty() {
            return None;
        }
        let mut s_end = s.len();
        loop {
            let mut start = s[..s_end].rfind(self.pattern())?;
            let newline = match (self.crlf, self.pattern.contains('\n')) {
                (true, true) if s[start..].starts_with('\n') && s[..start].ends_with('\r') => {
                    start -= 1;
                    Newline::CrLf
                }
                (true, false) if s[start..].starts_with("\r\n") => Newline::CrLf,
                _ => {
                    let Some(ch) = s[start..].chars().next() else {
                        unreachable!(
                        "Nonempty NewlineSet pattern should have matched at start of a character"
                    )
                    };
                    match Newline::try_from(ch) {
                        Ok(Newline::CarriageReturn) if !self.cr => {
                            s_end = start;
                            continue;
                        }
                        Ok(nl) => nl,
                        Err(_) => unreachable!(
                            "NewlineSet pattern should have matched a char that maps to Newline"
                        ),
                    }
                }
            };
            let end = start.saturating_add(newline.len_utf8());
            return Some(Match {
                start,
                end,
                newline,
                before: &s[..start],
                after: &s[end..],
            });
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct Match<'a> {
    pub start: usize,
    pub end: usize,
    pub newline: Newline,
    pub before: &'a str,
    pub after: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(Newline::LineFeed, "foobar", None)]
    #[case(Newline::LineFeed, "foo\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::LineFeed,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::LineFeed, "\nfoobar", Some(Match {
        start: 0,
        end: 1,
        newline: Newline::LineFeed,
        before: "",
        after: "foobar",
    }))]
    #[case(Newline::LineFeed, "foobar\n", Some(Match {
        start: 6,
        end: 7,
        newline: Newline::LineFeed,
        before: "foobar",
        after: "",
    }))]
    #[case(Newline::LineFeed, "foo\rbar", None)]
    #[case(Newline::CrLf, "foo\rbar", None)]
    #[case(Newline::CrLf, "foo\nbar", None)]
    #[case(Newline::CarriageReturn, "foo\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn, "foo\r\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "\nbar",
    }))]
    #[case(Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CrLf, "foo\rbar\r\nquux", Some(Match {
        start: 7,
        end: 9,
        newline: Newline::CrLf,
        before: "foo\rbar",
        after: "quux",
    }))]
    #[case(Newline::LineSeparator, "foo\u{2028}bar", Some(Match {
        start: 3,
        end: 6,
        newline: Newline::LineSeparator,
        before: "foo",
        after: "bar",
    }))]
    fn test_newline_search(
        #[case] nl: Newline,
        #[case] s: &'static str,
        #[case] m: Option<Match<'static>>,
    ) {
        assert_eq!(nl.search(s), m);
        if let Some(m) = m {
            assert_eq!(nl, m.newline);
            assert_eq!(&s[..m.start], m.before);
            assert_eq!(&s[m.end..], m.after);
            assert_eq!(&s[m.start..m.end], m.newline.as_str());
        }
    }

    #[rstest]
    #[case(NewlineSet::EMPTY, "foobar", None)]
    #[case(NewlineSet::EMPTY, "foo\r\nbar", None)]
    #[case(Newline::LineFeed.into(), "foobar", None)]
    #[case(Newline::LineFeed.into(), "foo\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::LineFeed,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::LineFeed.into(), "foo\r\nbar", Some(Match {
        start: 4,
        end: 5,
        newline: Newline::LineFeed,
        before: "foo\r",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar\r\nquux", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar\r\nquux",
    }))]
    #[case(Newline::CarriageReturn.into(), "foo\r\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "\nbar",
    }))]
    #[case(Newline::LineSeparator.into(), "foo\u{2028}bar", Some(Match {
        start: 3,
        end: 6,
        newline: Newline::LineSeparator,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::LineFeed | Newline::CarriageReturn, "foo\rbar\nquux", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar\nquux",
    }))]
    #[case(Newline::LineFeed | Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    fn test_newline_set_search(
        #[case] nlset: NewlineSet,
        #[case] s: &'static str,
        #[case] m: Option<Match<'static>>,
    ) {
        assert_eq!(nlset.search(s), m);
        if let Some(m) = m {
            assert!(nlset.contains(m.newline));
            assert_eq!(&s[..m.start], m.before);
            assert_eq!(&s[m.end..], m.after);
            assert_eq!(&s[m.start..m.end], m.newline.as_str());
        }
    }

    #[rstest]
    #[case(Newline::LineFeed, "foobar", None)]
    #[case(Newline::LineFeed, "foo\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::LineFeed,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::LineFeed, "\nfoobar", Some(Match {
        start: 0,
        end: 1,
        newline: Newline::LineFeed,
        before: "",
        after: "foobar",
    }))]
    #[case(Newline::LineFeed, "foobar\n", Some(Match {
        start: 6,
        end: 7,
        newline: Newline::LineFeed,
        before: "foobar",
        after: "",
    }))]
    #[case(Newline::LineFeed, "foo\rbar", None)]
    #[case(Newline::CrLf, "foo\rbar", None)]
    #[case(Newline::CrLf, "foo\nbar", None)]
    #[case(Newline::CarriageReturn, "foo\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn, "foo\r\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "\nbar",
    }))]
    #[case(Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CrLf, "foo\rbar\r\nquux", Some(Match {
        start: 7,
        end: 9,
        newline: Newline::CrLf,
        before: "foo\rbar",
        after: "quux",
    }))]
    #[case(Newline::LineSeparator, "foo\u{2028}bar", Some(Match {
        start: 3,
        end: 6,
        newline: Newline::LineSeparator,
        before: "foo",
        after: "bar",
    }))]
    fn test_newline_rsearch(
        #[case] nl: Newline,
        #[case] s: &'static str,
        #[case] m: Option<Match<'static>>,
    ) {
        assert_eq!(nl.rsearch(s), m);
        if let Some(m) = m {
            assert_eq!(nl, m.newline);
            assert_eq!(&s[..m.start], m.before);
            assert_eq!(&s[m.end..], m.after);
            assert_eq!(&s[m.start..m.end], m.newline.as_str());
        }
    }

    #[rstest]
    #[case(NewlineSet::ASCII, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::LineFeed | Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn.into(), "foo\r\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "\nbar",
    }))]
    #[case(Newline::CrLf.into(), "foo\r\nbar", Some(Match {
        start: 3,
        end: 5,
        newline: Newline::CrLf,
        before: "foo",
        after: "bar",
    }))]
    #[case(NewlineSet::ASCII, "foo\n\rbar", Some(Match {
        start: 4,
        end: 5,
        newline: Newline::CarriageReturn,
        before: "foo\n",
        after: "bar",
    }))]
    #[case(Newline::LineFeed | Newline::CrLf, "foo\n\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::LineFeed,
        before: "foo",
        after: "\rbar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\n\rbar", Some(Match {
        start: 4,
        end: 5,
        newline: Newline::CarriageReturn,
        before: "foo\n",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn.into(), "foo\n\rbar", Some(Match {
        start: 4,
        end: 5,
        newline: Newline::CarriageReturn,
        before: "foo\n",
        after: "bar",
    }))]
    #[case(Newline::CrLf.into(), "foo\n\rbar", None)]
    #[case(Newline::LineFeed | Newline::CrLf, "foo\nbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::LineFeed,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\nbar", None)]
    #[case(Newline::CarriageReturn.into(), "foo\nbar", None)]
    #[case(Newline::CrLf.into(), "foo\nbar", None)]
    #[case(Newline::LineFeed | Newline::CrLf, "foo\rbar", None)]
    #[case(Newline::CarriageReturn | Newline::CrLf, "foo\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CarriageReturn.into(), "foo\rbar", Some(Match {
        start: 3,
        end: 4,
        newline: Newline::CarriageReturn,
        before: "foo",
        after: "bar",
    }))]
    #[case(Newline::CrLf.into(), "foo\rbar", None)]
    #[case(Newline::LineSeparator.into(), "foo\u{2028}bar", Some(Match {
        start: 3,
        end: 6,
        newline: Newline::LineSeparator,
        before: "foo",
        after: "bar",
    }))]
    fn test_newline_set_rsearch(
        #[case] nlset: NewlineSet,
        #[case] s: &'static str,
        #[case] m: Option<Match<'static>>,
    ) {
        assert_eq!(nlset.rsearch(s), m);
        if let Some(m) = m {
            assert!(nlset.contains(m.newline));
            assert_eq!(&s[..m.start], m.before);
            assert_eq!(&s[m.end..], m.after);
            assert_eq!(&s[m.start..m.end], m.newline.as_str());
        }
    }
}
