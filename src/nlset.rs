use super::charset::{CharSet, Diff};
use super::iter::{
    AscendingNewlines, Difference, Intersection, IntoIter, SymmetricDifference, Union,
};
use super::nl::{CharType, Newline};
use std::fmt;
use std::ops;

#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct NewlineSet {
    /// The set of initial characters of the string representations of the
    /// `Newline` variants in the `NewlineSet`.  This is used to produce the
    /// `&[char]` pattern used to search strings for newlines.
    ///
    /// Note that `'\r'` will be in `pattern` if either or both of
    /// CarriageReturn and CrLf are in the `NewlineSet`.
    pub(crate) pattern: CharSet,

    /// Whether CarriageReturn is in the set
    pub(crate) cr: bool,

    /// Whether CrLf is in the set
    pub(crate) crlf: bool,
}

impl NewlineSet {
    // LF + CR LF
    pub const RUST: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\0', '\0', '\0', '\0', '\0'],
            len: 2,
        },
        cr: false,
        crlf: true,
    };

    // LF + CR LF + CR
    pub const ASCII: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\0', '\0', '\0', '\0', '\0'],
            len: 2,
        },
        cr: true,
        crlf: true,
    };

    // LF + CR LF + CR + NEL ("newline function" defined in §5.8 of the Unicode
    // Standard)
    pub const NLF: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\u{0085}', '\0', '\0', '\0', '\0'],
            len: 3,
        },
        cr: true,
        crlf: true,
    };

    // LF + CR LF + CR + NEL + FF + LineSep + ParaSep — Unicode's recommended
    // terminators when doing a "readline" operation, as per §5.8 of the
    // Unicode Standard
    pub const READLINE: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\x0C', '\r', '\u{0085}', '\u{2028}', '\u{2029}', '\0'],
            len: 6,
        },
        cr: true,
        crlf: true,
    };

    // READLINE | VerticalTab
    // Everything considered a newline by Unicode.  This will differ from ALL
    // if Python's splitlines()-specific endings are ever added to the library.
    pub const UNICODE: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: [
                '\n', '\x0B', '\x0C', '\r', '\u{0085}', '\u{2028}', '\u{2029}',
            ],
            len: 7,
        },
        cr: true,
        crlf: true,
    };

    pub const ALL: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: [
                '\n', '\x0B', '\x0C', '\r', '\u{0085}', '\u{2028}', '\u{2029}',
            ],
            len: 7,
        },
        cr: true,
        crlf: true,
    };

    pub const EMPTY: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\0'; 7],
            len: 0,
        },
        cr: false,
        crlf: false,
    };

    pub fn new() -> NewlineSet {
        NewlineSet::default()
    }

    pub fn len(&self) -> usize {
        self.pattern.len() + usize::from(self.cr && self.crlf)
    }

    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    pub fn contains(&self, nl: Newline) -> bool {
        match nl.chartype() {
            CharType::Char('\r') => self.cr,
            CharType::Char(ch) => self.pattern.contains(ch),
            CharType::CrLf => self.crlf,
        }
    }

    // Returns `true` if `nl` was not in `self` before insertion
    pub fn insert(&mut self, nl: Newline) -> bool {
        let ch = match nl.chartype() {
            CharType::Char('\r') => {
                if std::mem::replace(&mut self.cr, true) {
                    return false;
                }
                if self.crlf {
                    return true;
                }
                '\r'
            }
            CharType::Char(ch) => ch,
            CharType::CrLf => {
                if std::mem::replace(&mut self.crlf, true) {
                    return false;
                }
                if self.cr {
                    return true;
                }
                '\r'
            }
        };
        self.pattern.insert(ch)
    }

    // Returns `true` if `nl` was in `self` before removal
    pub fn remove(&mut self, nl: Newline) -> bool {
        let ch = match nl.chartype() {
            CharType::Char('\r') => {
                if !std::mem::replace(&mut self.cr, false) {
                    return false;
                }
                if self.crlf {
                    return true;
                }
                '\r'
            }
            CharType::Char(ch) => ch,
            CharType::CrLf => {
                if !std::mem::replace(&mut self.crlf, false) {
                    return false;
                }
                if self.cr {
                    return true;
                }
                '\r'
            }
        };
        self.pattern.remove(ch)
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_disjoint(&self, other: NewlineSet) -> bool {
        for d in self.pattern.diff(other.pattern) {
            match d {
                Diff::Left(_) | Diff::Right(_) => (),
                Diff::Both('\r') => {
                    if self.cr && other.cr || self.crlf && other.crlf {
                        return false;
                    }
                }
                Diff::Both(_) => return false,
            }
        }
        true
    }

    pub fn is_subset(&self, other: NewlineSet) -> bool {
        for d in self.pattern.diff(other.pattern) {
            match d {
                Diff::Both('\r') => {
                    if (self.cr && !other.cr) || (self.crlf && !other.crlf) {
                        return false;
                    }
                }
                Diff::Left(_) => return false,
                Diff::Right(_) | Diff::Both(_) => (),
            }
        }
        true
    }

    pub fn is_superset(&self, other: NewlineSet) -> bool {
        other.is_subset(*self)
    }

    pub fn union(self, other: NewlineSet) -> Union {
        Union::new(self, other)
    }

    pub fn intersection(self, other: NewlineSet) -> Intersection {
        Intersection::new(self, other)
    }

    pub fn symmetric_difference(self, other: NewlineSet) -> SymmetricDifference {
        SymmetricDifference::new(self, other)
    }

    pub fn difference(self, other: NewlineSet) -> Difference {
        Difference::new(self, other)
    }

    pub fn iter(&self) -> IntoIter {
        self.into_iter()
    }
}

impl fmt::Debug for NewlineSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(*self).finish()
    }
}

impl<T: Into<NewlineSet>> ops::BitAnd<T> for NewlineSet {
    type Output = NewlineSet;

    fn bitand(self, rhs: T) -> NewlineSet {
        self.intersection(rhs.into()).into_newline_set()
    }
}

impl<T: Into<NewlineSet>> ops::BitAnd<T> for Newline {
    type Output = NewlineSet;

    fn bitand(self, rhs: T) -> NewlineSet {
        if rhs.into().contains(self) {
            self.into()
        } else {
            NewlineSet::new()
        }
    }
}

impl<T: Into<NewlineSet>> ops::BitAndAssign<T> for NewlineSet {
    fn bitand_assign(&mut self, rhs: T) {
        *self = *self & rhs;
    }
}

impl<T: Into<NewlineSet>> ops::BitOr<T> for NewlineSet {
    type Output = NewlineSet;

    fn bitor(self, rhs: T) -> NewlineSet {
        self.union(rhs.into()).into_newline_set()
    }
}

impl<T: Into<NewlineSet>> ops::BitOr<T> for Newline {
    type Output = NewlineSet;

    fn bitor(self, rhs: T) -> NewlineSet {
        let mut nlset = rhs.into();
        nlset.insert(self);
        nlset
    }
}

impl<T: Into<NewlineSet>> ops::BitOrAssign<T> for NewlineSet {
    fn bitor_assign(&mut self, rhs: T) {
        *self = *self | rhs;
    }
}

impl<T: Into<NewlineSet>> ops::BitXor<T> for NewlineSet {
    type Output = NewlineSet;

    fn bitxor(self, rhs: T) -> NewlineSet {
        self.symmetric_difference(rhs.into()).into_newline_set()
    }
}

impl<T: Into<NewlineSet>> ops::BitXor<T> for Newline {
    type Output = NewlineSet;

    fn bitxor(self, rhs: T) -> NewlineSet {
        let mut nlset = rhs.into();
        if !nlset.remove(self) {
            nlset.insert(self);
        }
        nlset
    }
}

impl<T: Into<NewlineSet>> ops::BitXorAssign<T> for NewlineSet {
    fn bitxor_assign(&mut self, rhs: T) {
        *self = *self ^ rhs;
    }
}

impl<T: Into<NewlineSet>> ops::Sub<T> for NewlineSet {
    type Output = NewlineSet;

    fn sub(self, rhs: T) -> NewlineSet {
        self.difference(rhs.into()).into_newline_set()
    }
}

impl<T: Into<NewlineSet>> ops::Sub<T> for Newline {
    type Output = NewlineSet;

    fn sub(self, rhs: T) -> NewlineSet {
        if rhs.into().contains(self) {
            NewlineSet::new()
        } else {
            self.into()
        }
    }
}

impl<T: Into<NewlineSet>> ops::SubAssign<T> for NewlineSet {
    fn sub_assign(&mut self, rhs: T) {
        *self = *self - rhs;
    }
}

impl From<Newline> for NewlineSet {
    fn from(nl: Newline) -> NewlineSet {
        let mut nlset = NewlineSet::new();
        nlset.insert(nl);
        nlset
    }
}

impl<const N: usize> From<[Newline; N]> for NewlineSet {
    fn from(mut arr: [Newline; N]) -> NewlineSet {
        arr.sort_unstable();
        let mut nlset = NewlineSet::new();
        let mut prev_char = None;
        for nl in arr {
            let ch = match nl.chartype() {
                CharType::Char('\r') => {
                    nlset.cr = true;
                    '\r'
                }
                CharType::Char(ch) => ch,
                CharType::CrLf => {
                    nlset.crlf = true;
                    '\r'
                }
            };
            if prev_char.replace(ch) != Some(ch) {
                nlset.pattern.append(ch);
            }
        }
        nlset
    }
}

impl Extend<Newline> for NewlineSet {
    fn extend<I: IntoIterator<Item = Newline>>(&mut self, iter: I) {
        for nl in iter {
            self.insert(nl);
        }
    }
}

impl FromIterator<Newline> for NewlineSet {
    fn from_iter<I: IntoIterator<Item = Newline>>(iter: I) -> NewlineSet {
        iter.into_iter().fold(NewlineSet::new(), |mut nlset, nl| {
            nlset.insert(nl);
            nlset
        })
    }
}

impl IntoIterator for NewlineSet {
    type Item = Newline;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(self)
    }
}

impl IntoIterator for &NewlineSet {
    type Item = Newline;
    type IntoIter = IntoIter;

    fn into_iter(self) -> IntoIter {
        IntoIter::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;
    use rstest::rstest;

    fn assert_empty(nlset: NewlineSet) {
        assert_eq!(nlset.len(), 0);
        assert!(nlset.is_empty());
        for nl in Newline::iter() {
            assert!(!nlset.contains(nl));
        }
        assert!(!nlset.cr);
        assert!(!nlset.crlf);
        assert_eq!(nlset, NewlineSet::new());
        assert_eq!(nlset.into_iter().count(), 0);
    }

    fn assert_singleton(nlset: NewlineSet, nl: Newline) {
        assert_eq!(nlset.len(), 1);
        assert!(!nlset.is_empty());
        for nl2 in Newline::iter() {
            assert_eq!(nlset.contains(nl2), nl == nl2);
        }
        assert_eq!(nlset, NewlineSet::from([nl]));
        assert_eq!(nlset.into_iter().collect_vec(), [nl]);
    }

    fn assert_pair(nlset: NewlineSet, nl1: Newline, nl2: Newline) {
        assert_eq!(nlset.len(), 2);
        assert!(!nlset.is_empty());
        for nl in Newline::iter() {
            assert_eq!(nlset.contains(nl), nl == nl1 || nl == nl2);
        }
        assert_eq!(nlset, NewlineSet::from([nl1, nl2]));
        assert_eq!(
            nlset.into_iter().collect_vec(),
            [std::cmp::min(nl1, nl2), std::cmp::max(nl1, nl2)]
        );
    }

    #[test]
    fn test_empty() {
        let mut nlset = NewlineSet::new();
        assert_empty(nlset);
        nlset.clear();
        assert_empty(nlset);
    }

    #[test]
    fn test_insert_one() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl));
            assert_singleton(nlset, nl);
        }
    }

    #[test]
    fn test_insert_one_twice() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl));
            assert!(!nlset.insert(nl));
            assert_singleton(nlset, nl);
        }
    }

    #[test]
    fn test_insert_one_clear() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl));
            nlset.clear();
            assert_empty(nlset);
        }
    }

    #[test]
    fn test_insert_two() {
        for nls in Newline::iter().permutations(2) {
            let [nl1, nl2] = nls.try_into().unwrap();
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl1));
            assert!(nlset.insert(nl2));
            assert_pair(nlset, nl1, nl2);
        }
    }

    #[test]
    fn test_insert_two_clear() {
        for (nl1, nl2) in Newline::iter().tuple_combinations() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl1));
            assert!(nlset.insert(nl2));
            nlset.clear();
            assert_empty(nlset);
        }
    }

    #[test]
    fn test_remove_from_empty() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(!nlset.remove(nl));
            assert_empty(nlset);
        }
    }

    #[test]
    fn test_insert_and_remove() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl));
            assert!(nlset.remove(nl));
            assert_empty(nlset);
        }
    }

    #[test]
    fn test_insert_two_remove_first() {
        for nls in Newline::iter().permutations(2) {
            let [nl1, nl2] = nls.try_into().unwrap();
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl1));
            assert!(nlset.insert(nl2));
            assert!(nlset.remove(nl1));
            assert_singleton(nlset, nl2);
        }
    }

    #[test]
    fn test_from_newline() {
        for nl in Newline::iter() {
            let nlset = NewlineSet::from(nl);
            assert_singleton(nlset, nl);
        }
    }

    mod debug {
        use super::*;

        #[test]
        fn empty() {
            let nlset = NewlineSet::new();
            assert_eq!(format!("{nlset:?}"), "{}");
            assert_eq!(format!("{nlset:#?}"), "{}");
        }

        #[test]
        fn singleton() {
            let nlset = NewlineSet::from(Newline::LineFeed);
            assert_eq!(format!("{nlset:?}"), "{LineFeed}");
            assert_eq!(format!("{nlset:#?}"), "{\n    LineFeed,\n}");
        }

        #[test]
        fn pair() {
            let nlset = NewlineSet::from([Newline::LineFeed, Newline::CarriageReturn]);
            assert_eq!(format!("{nlset:?}"), "{LineFeed, CarriageReturn}");
            assert_eq!(
                format!("{nlset:#?}"),
                "{\n    LineFeed,\n    CarriageReturn,\n}"
            );
        }
    }

    mod from_array {
        use super::*;

        #[test]
        fn empty() {
            let nlset = NewlineSet::from([Newline::LineFeed; 0]);
            assert_empty(nlset);
        }

        #[test]
        fn singleton() {
            for nl in Newline::iter() {
                let nlset = NewlineSet::from([nl]);
                assert_singleton(nlset, nl);
            }
        }

        #[test]
        fn two_elem() {
            for nls in Newline::iter().permutations(2) {
                let [nl1, nl2] = nls.try_into().unwrap();
                let nlset = NewlineSet::from([nl1, nl2]);
                assert_pair(nlset, nl1, nl2);
            }
        }

        #[test]
        fn duplicated_elem() {
            for nl in Newline::iter() {
                let nlset = NewlineSet::from([nl, nl]);
                assert_singleton(nlset, nl);
            }
        }
    }

    mod from_iterator {
        use super::*;

        #[test]
        fn empty() {
            let nlset = NewlineSet::from_iter(std::iter::empty());
            assert_empty(nlset);
        }

        #[test]
        fn singleton() {
            for nl in Newline::iter() {
                let nlset = NewlineSet::from_iter(std::iter::once(nl));
                assert_singleton(nlset, nl);
            }
        }

        #[test]
        fn two_elem() {
            for nls in Newline::iter().permutations(2) {
                let nl1 = nls[0];
                let nl2 = nls[1];
                let nlset = NewlineSet::from_iter(nls);
                assert_pair(nlset, nl1, nl2);
            }
        }

        #[test]
        fn duplicated_elem() {
            for nl in Newline::iter() {
                let nlset = NewlineSet::from_iter([nl, nl]);
                assert_singleton(nlset, nl);
            }
        }

        #[test]
        fn all() {
            let nlset = NewlineSet::from_iter(Newline::iter());
            assert_eq!(nlset.len(), Newline::COUNT);
            assert!(!nlset.is_empty());
            for nl in Newline::iter() {
                assert!(nlset.contains(nl));
            }
            assert_eq!(
                nlset.into_iter().collect_vec(),
                Newline::iter().collect_vec()
            );
        }
    }

    mod extend {
        use super::*;

        #[test]
        fn empty_plus_empty() {
            let mut nlset = NewlineSet::new();
            nlset.extend(std::iter::empty());
            assert_empty(nlset);
        }

        #[test]
        fn empty_plus_one() {
            for nl in Newline::iter() {
                let mut nlset = NewlineSet::new();
                nlset.extend([nl]);
                assert_singleton(nlset, nl);
            }
        }

        #[test]
        fn empty_plus_two() {
            for nls in Newline::iter().permutations(2) {
                let nl1 = nls[0];
                let nl2 = nls[1];
                let mut nlset = NewlineSet::new();
                nlset.extend(nls);
                assert_pair(nlset, nl1, nl2);
            }
        }

        #[test]
        fn empty_plus_one_twice() {
            for nl in Newline::iter() {
                let mut nlset = NewlineSet::new();
                nlset.extend([nl, nl]);
                assert_singleton(nlset, nl);
            }
        }

        #[test]
        fn empty_plus_all() {
            let mut nlset = NewlineSet::new();
            nlset.extend(Newline::iter());
            assert_eq!(nlset.len(), Newline::COUNT);
            assert!(!nlset.is_empty());
            for nl in Newline::iter() {
                assert!(nlset.contains(nl));
            }
            assert_eq!(
                nlset.into_iter().collect_vec(),
                Newline::iter().collect_vec()
            );
        }

        #[test]
        fn mixture() {
            let mut nlset =
                NewlineSet::from([Newline::LineFeed, Newline::CarriageReturn, Newline::CrLf]);
            nlset.extend([Newline::VerticalTab, Newline::LineFeed, Newline::NextLine]);
            assert_eq!(nlset.len(), 5);
            assert!(!nlset.is_empty());
            assert_eq!(
                nlset,
                NewlineSet::from([
                    Newline::LineFeed,
                    Newline::VerticalTab,
                    Newline::CarriageReturn,
                    Newline::CrLf,
                    Newline::NextLine,
                ])
            );
            assert_eq!(
                nlset.into_iter().collect_vec(),
                [
                    Newline::LineFeed,
                    Newline::VerticalTab,
                    Newline::CarriageReturn,
                    Newline::CrLf,
                    Newline::NextLine,
                ]
            );
        }
    }

    mod is_disjoint {
        use super::*;

        #[test]
        fn empty_empty() {
            let nlset1 = NewlineSet::new();
            let nlset2 = NewlineSet::new();
            assert!(nlset1.is_disjoint(nlset2));
            assert!(nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn empty_singleton() {
            let nlset1 = NewlineSet::new();
            let nlset2 = NewlineSet::from(Newline::LineFeed);
            assert!(nlset1.is_disjoint(nlset2));
            assert!(nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn different_singletons() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from(Newline::LineFeed);
            assert!(nlset1.is_disjoint(nlset2));
            assert!(nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn equal_singletons() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from(Newline::FormFeed);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn overlapping_pairs() {
            let nlset1 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            let nlset2 = NewlineSet::from([Newline::VerticalTab, Newline::FormFeed]);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn singleton_vs_more() {
            let nlset1 = NewlineSet::from(Newline::LineFeed);
            let nlset2 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn singleton_vs_unrelated() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            assert!(nlset1.is_disjoint(nlset2));
            assert!(nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn cr_vs_cr() {
            let nlset1 = NewlineSet::from(Newline::CarriageReturn);
            let nlset2 = NewlineSet::from(Newline::CarriageReturn);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn cr_vs_crlf() {
            let nlset1 = NewlineSet::from(Newline::CarriageReturn);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(nlset1.is_disjoint(nlset2));
            assert!(nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn crlf_vs_crlf() {
            let nlset1 = NewlineSet::from(Newline::CrLf);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn cr_crlf_vs_cr() {
            let nlset1 = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf]);
            let nlset2 = NewlineSet::from(Newline::CarriageReturn);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }

        #[test]
        fn cr_crlf_vs_crlf() {
            let nlset1 = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf]);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(!nlset1.is_disjoint(nlset2));
            assert!(!nlset2.is_disjoint(nlset1));
        }
    }

    mod subset_superset {
        use super::*;

        #[test]
        fn empty_empty() {
            let nlset1 = NewlineSet::new();
            let nlset2 = NewlineSet::new();
            assert!(nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn empty_singleton() {
            let nlset1 = NewlineSet::new();
            let nlset2 = NewlineSet::from(Newline::LineFeed);
            assert!(nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn different_singletons() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from(Newline::LineFeed);
            assert!(!nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }

        #[test]
        fn equal_singletons() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from(Newline::FormFeed);
            assert!(nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn overlapping_pairs() {
            let nlset1 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            let nlset2 = NewlineSet::from([Newline::VerticalTab, Newline::FormFeed]);
            assert!(!nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }

        #[test]
        fn singleton_vs_more() {
            let nlset1 = NewlineSet::from(Newline::LineFeed);
            let nlset2 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            assert!(nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn singleton_vs_unrelated() {
            let nlset1 = NewlineSet::from(Newline::FormFeed);
            let nlset2 = NewlineSet::from([Newline::LineFeed, Newline::VerticalTab]);
            assert!(!nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }

        #[test]
        fn cr_vs_cr() {
            let nlset1 = NewlineSet::from(Newline::CarriageReturn);
            let nlset2 = NewlineSet::from(Newline::CarriageReturn);
            assert!(nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn cr_vs_crlf() {
            let nlset1 = NewlineSet::from(Newline::CarriageReturn);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(!nlset1.is_subset(nlset2));
            assert!(!nlset2.is_subset(nlset1));
            assert!(!nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }

        #[test]
        fn crlf_vs_crlf() {
            let nlset1 = NewlineSet::from(Newline::CrLf);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(nlset2.is_superset(nlset1));
        }

        #[test]
        fn cr_crlf_vs_cr() {
            let nlset1 = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf]);
            let nlset2 = NewlineSet::from(Newline::CarriageReturn);
            assert!(!nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }

        #[test]
        fn cr_crlf_vs_crlf() {
            let nlset1 = NewlineSet::from([Newline::CarriageReturn, Newline::CrLf]);
            let nlset2 = NewlineSet::from(Newline::CrLf);
            assert!(!nlset1.is_subset(nlset2));
            assert!(nlset2.is_subset(nlset1));
            assert!(nlset1.is_superset(nlset2));
            assert!(!nlset2.is_superset(nlset1));
        }
    }

    mod consts {
        use super::*;

        #[test]
        fn rust() {
            assert_eq!(
                NewlineSet::RUST,
                NewlineSet::from([Newline::LineFeed, Newline::CrLf])
            );
        }

        #[test]
        fn ascii() {
            assert_eq!(
                NewlineSet::ASCII,
                NewlineSet::from([Newline::LineFeed, Newline::CarriageReturn, Newline::CrLf])
            );
        }

        #[test]
        fn nlf() {
            assert_eq!(
                NewlineSet::NLF,
                NewlineSet::from([
                    Newline::LineFeed,
                    Newline::CarriageReturn,
                    Newline::CrLf,
                    Newline::NextLine
                ])
            );
        }

        #[test]
        fn readline() {
            assert_eq!(
                NewlineSet::READLINE,
                NewlineSet::from([
                    Newline::LineFeed,
                    Newline::FormFeed,
                    Newline::CarriageReturn,
                    Newline::CrLf,
                    Newline::NextLine,
                    Newline::LineSeparator,
                    Newline::ParagraphSeparator
                ])
            );
        }

        #[test]
        fn unicode() {
            assert_eq!(
                NewlineSet::UNICODE,
                NewlineSet::from([
                    Newline::LineFeed,
                    Newline::VerticalTab,
                    Newline::FormFeed,
                    Newline::CarriageReturn,
                    Newline::CrLf,
                    Newline::NextLine,
                    Newline::LineSeparator,
                    Newline::ParagraphSeparator
                ])
            );
        }

        #[test]
        fn all() {
            assert_eq!(NewlineSet::ALL, NewlineSet::from_iter(Newline::iter()));
        }

        #[test]
        fn empty() {
            assert_eq!(NewlineSet::EMPTY, NewlineSet::new());
        }
    }

    #[rstest]
    #[case(Vec::new(), Vec::new(), Vec::new())]
    #[case(Vec::new(), vec![Newline::LineFeed], vec![Newline::LineFeed])]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn],
        vec![Newline::CarriageReturn],
    )]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(Vec::new(), vec![Newline::CrLf], vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        vec![Newline::LineFeed],
        vec![Newline::CarriageReturn],
        vec![Newline::LineFeed, Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::LineFeed, Newline::FormFeed],
        vec![Newline::FormFeed, Newline::VerticalTab],
        vec![Newline::LineFeed, Newline::VerticalTab, Newline::FormFeed],
    )]
    #[case(
        Newline::iter().collect(),
        vec![Newline::NextLine],
        Newline::iter().collect(),
    )]
    fn test_union(
        #[case] left: Vec<Newline>,
        #[case] right: Vec<Newline>,
        #[case] both: Vec<Newline>,
    ) {
        let nlset1 = NewlineSet::from_iter(left);
        let nlset2 = NewlineSet::from_iter(right);
        assert_eq!(nlset1.union(nlset2).collect_vec(), both);
        assert_eq!(nlset2.union(nlset1).collect_vec(), both);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset1 | nlset2, combo);
        assert_eq!(nlset2 | nlset1, combo);
        let mut agg1 = nlset1;
        agg1 |= nlset2;
        assert_eq!(agg1, combo);
        let mut agg2 = nlset2;
        agg2 |= nlset1;
        assert_eq!(agg2, combo);
    }

    #[rstest]
    #[case(Vec::new(), Newline::LineFeed, vec![Newline::LineFeed])]
    #[case(Vec::new(), Newline::CarriageReturn, vec![Newline::CarriageReturn])]
    #[case(Vec::new(), Newline::CrLf, vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn],
        Newline::CrLf,
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        vec![Newline::LineFeed],
        Newline::CarriageReturn,
        vec![Newline::LineFeed, Newline::CarriageReturn],
    )]
    #[case(
        Newline::iter().collect(),
        Newline::NextLine,
        Newline::iter().collect(),
    )]
    fn nlset_bitor_nl(#[case] left: Vec<Newline>, #[case] nl: Newline, #[case] both: Vec<Newline>) {
        let mut nlset = NewlineSet::from_iter(left);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset | nl, combo);
        assert_eq!(nl | nlset, combo);
        nlset |= nl;
        assert_eq!(nlset, combo);
    }

    #[rstest]
    #[case(Newline::LineFeed, Newline::LineFeed, vec![Newline::LineFeed])]
    #[case(Newline::LineFeed, Newline::CarriageReturn, vec![Newline::LineFeed, Newline::CarriageReturn])]
    #[case(Newline::LineFeed, Newline::CrLf, vec![Newline::LineFeed, Newline::CrLf])]
    #[case(Newline::CrLf, Newline::CrLf, vec![Newline::CrLf])]
    #[case(Newline::CarriageReturn, Newline::CarriageReturn, vec![Newline::CarriageReturn])]
    #[case(Newline::CarriageReturn, Newline::CrLf, vec![Newline::CarriageReturn, Newline::CrLf])]
    #[case(Newline::LineFeed, Newline::NextLine, vec![Newline::LineFeed, Newline::NextLine])]
    fn nl_bitor_nl(#[case] nl1: Newline, #[case] nl2: Newline, #[case] both: Vec<Newline>) {
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nl1 | nl2, combo);
        assert_eq!(nl2 | nl1, combo);
    }

    #[rstest]
    #[case(Vec::new(), Vec::new(), Vec::new())]
    #[case(Vec::new(), vec![Newline::LineFeed], Vec::new())]
    #[case(Vec::new(), vec![Newline::CarriageReturn], Vec::new())]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(Vec::new(), vec![Newline::CrLf], Vec::new())]
    #[case(vec![Newline::CarriageReturn], vec![Newline::CrLf], Vec::new())]
    #[case(vec![Newline::LineFeed], vec![Newline::CarriageReturn], Vec::new())]
    #[case(vec![Newline::LineFeed], vec![Newline::LineFeed], vec![Newline::LineFeed])]
    #[case(
        vec![Newline::LineFeed, Newline::FormFeed],
        vec![Newline::FormFeed, Newline::VerticalTab],
        vec![Newline::FormFeed],
    )]
    #[case(
        Newline::iter().collect(),
        vec![Newline::NextLine],
        vec![Newline::NextLine],
    )]
    fn test_intersection(
        #[case] left: Vec<Newline>,
        #[case] right: Vec<Newline>,
        #[case] both: Vec<Newline>,
    ) {
        let nlset1 = NewlineSet::from_iter(left);
        let nlset2 = NewlineSet::from_iter(right);
        assert_eq!(nlset1.intersection(nlset2).collect_vec(), both);
        assert_eq!(nlset2.intersection(nlset1).collect_vec(), both);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset1 & nlset2, combo);
        assert_eq!(nlset2 & nlset1, combo);
        let mut agg1 = nlset1;
        agg1 &= nlset2;
        assert_eq!(agg1, combo);
        let mut agg2 = nlset2;
        agg2 &= nlset1;
        assert_eq!(agg2, combo);
    }

    #[rstest]
    #[case(Vec::new(), Newline::LineFeed, Vec::new())]
    #[case(Vec::new(), Newline::CarriageReturn, Vec::new())]
    #[case(Vec::new(), Newline::CrLf, Vec::new())]
    #[case(vec![Newline::CarriageReturn], Newline::CrLf, Vec::new())]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CrLf,
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CarriageReturn,
        vec![Newline::CarriageReturn],
    )]
    #[case(vec![Newline::LineFeed], Newline::CarriageReturn, Vec::new())]
    #[case(vec![Newline::LineFeed], Newline::LineFeed, vec![Newline::LineFeed])]
    #[case(
        Newline::iter().collect(),
        Newline::NextLine,
        vec![Newline::NextLine],
    )]
    fn nlset_bitand_nl(
        #[case] left: Vec<Newline>,
        #[case] nl: Newline,
        #[case] both: Vec<Newline>,
    ) {
        let mut nlset = NewlineSet::from_iter(left);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset & nl, combo);
        assert_eq!(nl & nlset, combo);
        nlset &= nl;
        assert_eq!(nlset, combo);
    }

    #[rstest]
    #[case(Newline::LineFeed, Newline::LineFeed, vec![Newline::LineFeed])]
    #[case(Newline::LineFeed, Newline::CarriageReturn, Vec::new())]
    #[case(Newline::LineFeed, Newline::CrLf, Vec::new())]
    #[case(Newline::CrLf, Newline::CrLf, vec![Newline::CrLf])]
    #[case(Newline::CarriageReturn, Newline::CarriageReturn, vec![Newline::CarriageReturn])]
    #[case(Newline::CarriageReturn, Newline::CrLf, Vec::new())]
    #[case(Newline::LineFeed, Newline::NextLine, Vec::new())]
    fn nl_bitand_nl(#[case] nl1: Newline, #[case] nl2: Newline, #[case] both: Vec<Newline>) {
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nl1 & nl2, combo);
        assert_eq!(nl2 & nl1, combo);
    }

    #[rstest]
    #[case(Vec::new(), Vec::new(), Vec::new())]
    #[case(Vec::new(), vec![Newline::LineFeed], vec![Newline::LineFeed])]
    #[case(
        vec![Newline::LineFeed],
        vec![Newline::NextLine],
        vec![Newline::LineFeed, Newline::NextLine],
    )]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn],
        vec![Newline::CarriageReturn],
    )]
    #[case(Vec::new(), vec![Newline::CrLf], vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::LineFeed],
        vec![Newline::CarriageReturn],
        vec![Newline::LineFeed, Newline::CarriageReturn],
    )]
    #[case(vec![Newline::LineFeed], vec![Newline::LineFeed], Vec::new())]
    #[case(
        vec![Newline::LineFeed, Newline::FormFeed],
        vec![Newline::FormFeed, Newline::VerticalTab],
        vec![Newline::LineFeed, Newline::VerticalTab],
    )]
    #[case(
        Newline::iter().collect(),
        vec![Newline::NextLine],
        Newline::iter().filter(|&nl| nl != Newline::NextLine).collect(),
    )]
    fn test_symmetric_difference(
        #[case] left: Vec<Newline>,
        #[case] right: Vec<Newline>,
        #[case] both: Vec<Newline>,
    ) {
        let nlset1 = NewlineSet::from_iter(left);
        let nlset2 = NewlineSet::from_iter(right);
        assert_eq!(nlset1.symmetric_difference(nlset2).collect_vec(), both);
        assert_eq!(nlset2.symmetric_difference(nlset1).collect_vec(), both);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset1 ^ nlset2, combo);
        assert_eq!(nlset2 ^ nlset1, combo);
        let mut agg1 = nlset1;
        agg1 ^= nlset2;
        assert_eq!(agg1, combo);
        let mut agg2 = nlset2;
        agg2 ^= nlset1;
        assert_eq!(agg2, combo);
    }

    #[rstest]
    #[case(Vec::new(), Newline::LineFeed, vec![Newline::LineFeed])]
    #[case(Vec::new(), Newline::CarriageReturn, vec![Newline::CarriageReturn])]
    #[case(Vec::new(), Newline::CrLf, vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn],
        Newline::CrLf,
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CrLf,
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CarriageReturn,
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::LineFeed],
        Newline::CarriageReturn,
        vec![Newline::LineFeed, Newline::CarriageReturn],
    )]
    #[case(vec![Newline::LineFeed], Newline::LineFeed, Vec::new())]
    #[case(
        Newline::iter().collect(),
        Newline::NextLine,
        Newline::iter().filter(|&nl| nl != Newline::NextLine).collect(),
    )]
    fn nlset_bitxor_nl(
        #[case] left: Vec<Newline>,
        #[case] nl: Newline,
        #[case] both: Vec<Newline>,
    ) {
        let mut nlset = NewlineSet::from_iter(left);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset ^ nl, combo);
        assert_eq!(nl ^ nlset, combo);
        nlset ^= nl;
        assert_eq!(nlset, combo);
    }

    #[rstest]
    #[case(Newline::LineFeed, Newline::LineFeed, Vec::new())]
    #[case(Newline::LineFeed, Newline::CarriageReturn, vec![Newline::LineFeed, Newline::CarriageReturn])]
    #[case(Newline::LineFeed, Newline::CrLf, vec![Newline::LineFeed, Newline::CrLf])]
    #[case(Newline::CrLf, Newline::CrLf, Vec::new())]
    #[case(Newline::CarriageReturn, Newline::CarriageReturn, Vec::new())]
    #[case(Newline::CarriageReturn, Newline::CrLf, vec![Newline::CarriageReturn, Newline::CrLf])]
    #[case(Newline::LineFeed, Newline::NextLine, vec![Newline::LineFeed, Newline::NextLine])]
    fn nl_bitxor_nl(#[case] nl1: Newline, #[case] nl2: Newline, #[case] both: Vec<Newline>) {
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nl1 ^ nl2, combo);
        assert_eq!(nl2 ^ nl1, combo);
    }

    #[rstest]
    #[case(Vec::new(), Vec::new(), Vec::new())]
    #[case(Vec::new(), vec![Newline::LineFeed], Vec::new())]
    #[case(vec![Newline::LineFeed], Vec::new(), vec![Newline::LineFeed])]
    #[case(
        vec![Newline::LineFeed],
        vec![Newline::NextLine],
        vec![Newline::LineFeed],
    )]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn],
        Vec::new(),
    )]
    #[case(
        vec![Newline::CarriageReturn],
        Vec::new(),
        vec![Newline::CarriageReturn],
    )]
    #[case(Vec::new(), vec![Newline::CrLf], Vec::new())]
    #[case(vec![Newline::CrLf], Vec::new(), vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn],
        vec![Newline::CrLf],
    )]
    #[case(
        Vec::new(),
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
        vec![Newline::CarriageReturn, Newline::CrLf],
    )]
    #[case(
        vec![Newline::CarriageReturn],
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn],
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CrLf],
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![Newline::CarriageReturn, Newline::CrLf],
        Vec::new(),
    )]
    #[case(
        vec![Newline::LineFeed],
        vec![Newline::CarriageReturn],
        vec![Newline::LineFeed],
    )]
    #[case(vec![Newline::LineFeed], vec![Newline::LineFeed], Vec::new())]
    #[case(
        vec![Newline::LineFeed, Newline::FormFeed],
        vec![Newline::FormFeed, Newline::VerticalTab],
        vec![Newline::LineFeed],
    )]
    #[case(
        Newline::iter().collect(),
        vec![Newline::NextLine],
        Newline::iter().filter(|&nl| nl != Newline::NextLine).collect(),
    )]
    #[case(
        vec![Newline::NextLine],
        Newline::iter().collect(),
        Vec::new(),
    )]
    fn test_difference(
        #[case] left: Vec<Newline>,
        #[case] right: Vec<Newline>,
        #[case] both: Vec<Newline>,
    ) {
        let nlset1 = NewlineSet::from_iter(left);
        let nlset2 = NewlineSet::from_iter(right);
        assert_eq!(nlset1.difference(nlset2).collect_vec(), both);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset1 - nlset2, combo);
        let mut agg = nlset1;
        agg -= nlset2;
        assert_eq!(agg, combo);
    }

    #[rstest]
    #[case(Vec::new(), Newline::LineFeed, Vec::new())]
    #[case(vec![Newline::LineFeed], Newline::NextLine, vec![Newline::LineFeed])]
    #[case(Vec::new(), Newline::CarriageReturn, Vec::new())]
    #[case(Vec::new(), Newline::CrLf, Vec::new())]
    #[case(
        vec![Newline::CarriageReturn],
        Newline::CrLf,
        vec![Newline::CarriageReturn],
    )]
    #[case(vec![Newline::CrLf], Newline::CarriageReturn, vec![Newline::CrLf])]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CrLf,
        vec![Newline::CarriageReturn],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        Newline::CarriageReturn,
        vec![Newline::CrLf],
    )]
    #[case(
        vec![Newline::LineFeed],
        Newline::CarriageReturn,
        vec![Newline::LineFeed],
    )]
    #[case(vec![Newline::LineFeed], Newline::LineFeed, Vec::new())]
    #[case(
        Newline::iter().collect(),
        Newline::NextLine,
        Newline::iter().filter(|&nl| nl != Newline::NextLine).collect(),
    )]
    fn nlset_sub_nl(#[case] left: Vec<Newline>, #[case] nl: Newline, #[case] both: Vec<Newline>) {
        let mut nlset = NewlineSet::from_iter(left);
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nlset - nl, combo);
        nlset -= nl;
        assert_eq!(nlset, combo);
    }

    #[rstest]
    #[case(Newline::LineFeed, Newline::LineFeed, Vec::new())]
    #[case(Newline::LineFeed, Newline::CarriageReturn, vec![Newline::LineFeed])]
    #[case(Newline::LineFeed, Newline::CrLf, vec![Newline::LineFeed])]
    #[case(Newline::CrLf, Newline::CrLf, Vec::new())]
    #[case(Newline::CarriageReturn, Newline::CarriageReturn, Vec::new())]
    #[case(Newline::CarriageReturn, Newline::CrLf, vec![Newline::CarriageReturn])]
    #[case(Newline::LineFeed, Newline::NextLine, vec![Newline::LineFeed])]
    fn nl_sub_nl(#[case] nl1: Newline, #[case] nl2: Newline, #[case] both: Vec<Newline>) {
        let combo = NewlineSet::from_iter(both);
        assert_eq!(nl1 - nl2, combo);
    }
}
