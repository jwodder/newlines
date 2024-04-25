use super::charset::{CharSet, Diff};
use super::iter::IntoIter;
use super::nl::{CharType, Newline};
use std::fmt;

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
}

impl fmt::Debug for NewlineSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(*self).finish()
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

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

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
}
