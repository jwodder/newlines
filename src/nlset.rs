use super::charset::{CharSet, Diff};
use super::iter::{
    AscendingNewlines, Complement, Difference, Intersection, IntoIter, SymmetricDifference, Union,
};
use super::nl::{CharType, Newline};
use core::fmt;
use core::ops;

/// A set of newline sequences that can be used to search for or split on any
/// sequence in the set.
///
/// A `NewlineSet` can be constructed in the following ways:
///
/// - By creating an empty set with [`NewlineSet::new()`]
///
/// - By converting a single [`Newline`] to a `NewlineSet` containing only that
///   newline using the [`From`] & [`Into`] traits
///
/// - By applying `Newline::from()` to an array of [`Newline`] values
///
/// - By applying `Newline::from_iter()` to an `IntoIterator` type that yields
///   [`Newline`] values
///
/// Constants are also provided for common collections of newline sequences.
///
/// `NewlineSet` values can be combined using the following operators:
///
/// - `|` and `|=` for union
/// - `&` and `&=` for intersection
/// - `^` and `^=` for symmetric difference
/// - `-` and `-=` for difference
///
/// For the non-assignment operators, either side may be a `Newline` or
/// `NewlineSet`.  For the assignment operators, the right-hand side may be a
/// `Newline` or `NewlineSet`.
///
/// The complement of a `NewlineSet` can also be acquired by applying the `!`
/// operator to it.
#[derive(Copy, Clone, Default, Eq, Hash, PartialEq)]
pub struct NewlineSet {
    /// The set of initial characters of the string representations of the
    /// `Newline` variants in the `NewlineSet`.  This is used to produce the
    /// `&[char]` pattern used to search strings for newlines.
    ///
    /// Note that `'\r'` will be in `pattern` if either or both of
    /// `CarriageReturn` and `CrLf` are in the `NewlineSet`.
    pub(crate) pattern: CharSet,

    /// Whether `CarriageReturn` is in the set
    pub(crate) cr: bool,

    /// Whether `CrLf` is in the set
    pub(crate) crlf: bool,
}

impl NewlineSet {
    /// An empty `NewlineSet`
    pub const EMPTY: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\0'; 7],
            len: 0,
        },
        cr: false,
        crlf: false,
    };

    /// The newline sequences recognized by Rust's standard library in methods
    /// like [`str::lines()`] and [`std::io::BufRead::lines()`]:
    ///
    /// - [`Newline::LineFeed`]
    /// - [`Newline::CrLf`]
    pub const RUST: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\0', '\0', '\0', '\0', '\0'],
            len: 2,
        },
        cr: false,
        crlf: true,
    };

    /// The typical ASCII newline sequences:
    ///
    /// - [`Newline::LineFeed`]
    /// - [`Newline::CarriageReturn`]
    /// - [`Newline::CrLf`]
    pub const ASCII: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\0', '\0', '\0', '\0', '\0'],
            len: 2,
        },
        cr: true,
        crlf: true,
    };

    /// Newline sequences classified as "newline functions" by Unicode §5.8,
    /// "Newline Guidelines":
    ///
    /// - [`Newline::LineFeed`]
    /// - [`Newline::CarriageReturn`]
    /// - [`Newline::CrLf`]
    /// - [`Newline::NextLine`]
    pub const NLF: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\r', '\u{0085}', '\0', '\0', '\0', '\0'],
            len: 3,
        },
        cr: true,
        crlf: true,
    };

    /// Newline sequences that Unicode §5.8, "Newline Guidelines", advises
    /// readline functions to stop at:
    ///
    /// - [`Newline::LineFeed`]
    /// - [`Newline::FormFeed`]
    /// - [`Newline::CarriageReturn`]
    /// - [`Newline::CrLf`]
    /// - [`Newline::NextLine`]
    /// - [`Newline::LineSeparator`]
    /// - [`Newline::ParagraphSeparator`]
    pub const READLINE: NewlineSet = NewlineSet {
        pattern: CharSet {
            data: ['\n', '\x0C', '\r', '\u{0085}', '\u{2028}', '\u{2029}', '\0'],
            len: 6,
        },
        cr: true,
        crlf: true,
    };

    /// All newline sequences that are treated as mandatory line breaks by the
    /// [Unicode Line Breaking Algorithm][tr14]:
    ///
    /// - [`Newline::LineFeed`]
    /// - [`Newline::VerticalTab`]
    /// - [`Newline::FormFeed`]
    /// - [`Newline::CarriageReturn`]
    /// - [`Newline::CrLf`]
    /// - [`Newline::NextLine`]
    /// - [`Newline::LineSeparator`]
    /// - [`Newline::ParagraphSeparator`]
    ///
    /// [tr14]: https://www.unicode.org/reports/tr14/
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

    /// All newline sequences recognized by this library
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

    /// Create an empty `NewlineSet`
    pub fn new() -> NewlineSet {
        NewlineSet::default()
    }

    /// Returns the number of [`Newline`] variants in the set
    pub fn len(&self) -> usize {
        self.pattern.len() + usize::from(self.cr && self.crlf)
    }

    /// Returns `true` if the set is empty
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    /// [Private] Returns a `char` slice that can be used to search a string
    /// for any occurrence of a character that starts a newline sequence in the
    /// set
    pub(crate) fn pattern(&self) -> &[char] {
        self.pattern.as_slice()
    }

    /// Returns `true` if the given [`Newline`] is in the set
    pub fn contains(&self, nl: Newline) -> bool {
        match nl.chartype() {
            CharType::Char('\r') => self.cr,
            CharType::Char(ch) => self.pattern.contains(ch),
            CharType::CrLf => self.crlf,
        }
    }

    /// Adds the given [`Newline`] to the set if not already present.
    ///
    /// Returns `true` if the given `Newline` was not already in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use newlines::{Newline, NewlineSet};
    ///
    /// let mut nlset = NewlineSet::new();
    /// assert!(nlset.insert(Newline::LineFeed));
    /// assert!(!nlset.insert(Newline::LineFeed));
    /// assert!(nlset.insert(Newline::CrLf));
    /// ```
    pub fn insert(&mut self, nl: Newline) -> bool {
        let ch = match nl.chartype() {
            CharType::Char('\r') => {
                if core::mem::replace(&mut self.cr, true) {
                    return false;
                }
                if self.crlf {
                    return true;
                }
                '\r'
            }
            CharType::Char(ch) => ch,
            CharType::CrLf => {
                if core::mem::replace(&mut self.crlf, true) {
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

    /// Removes the given [`Newline`] from the set if present.
    ///
    /// Returns `true` if the given `Newline` was present in the set.
    ///
    /// # Example
    ///
    /// ```
    /// use newlines::Newline;
    ///
    /// let mut nlset = Newline::LineFeed | Newline::CrLf;
    /// assert!(nlset.remove(Newline::LineFeed));
    /// assert!(!nlset.remove(Newline::LineFeed));
    /// assert!(nlset.remove(Newline::CrLf));
    /// assert!(!nlset.remove(Newline::NextLine));
    /// ```
    pub fn remove(&mut self, nl: Newline) -> bool {
        let ch = match nl.chartype() {
            CharType::Char('\r') => {
                if !core::mem::replace(&mut self.cr, false) {
                    return false;
                }
                if self.crlf {
                    return true;
                }
                '\r'
            }
            CharType::Char(ch) => ch,
            CharType::CrLf => {
                if !core::mem::replace(&mut self.crlf, false) {
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

    /// Removes all [`Newline`]s from the set
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// Returns true if `self` and `other` are disjoint, i.e., if there is no
    /// [`Newline`] variant that is in both sets.
    ///
    /// # Example
    ///
    /// ```
    /// use newlines::Newline;
    ///
    /// let nlset1 = Newline::LineFeed | Newline::CrLf;
    /// let nlset2 = Newline::LineSeparator | Newline::ParagraphSeparator;
    /// let nlset3 = Newline::CarriageReturn | Newline::CrLf;
    /// assert!(nlset1.is_disjoint(nlset2));
    /// assert!(!nlset1.is_disjoint(nlset3));
    /// ```
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

    /// Returns `true` if `self` is a subset of `other`
    ///
    /// # Example
    ///
    /// ```
    /// use newlines::Newline;
    ///
    /// let nlset1 = Newline::LineFeed | Newline::CrLf;
    /// let nlset2 = Newline::LineFeed | Newline::CrLf | Newline::CarriageReturn;
    /// let nlset3 = Newline::LineFeed | Newline::CarriageReturn;
    /// assert!(nlset1.is_subset(nlset2));
    /// assert!(!nlset1.is_subset(nlset3));
    /// ```
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

    /// Returns `true` if `self` is a superset of `other`
    ///
    /// # Example
    ///
    /// ```
    /// use newlines::Newline;
    ///
    /// let nlset1 = Newline::LineFeed | Newline::CrLf | Newline::CarriageReturn;
    /// let nlset2 = Newline::LineFeed | Newline::CrLf;
    /// let nlset3 = Newline::LineFeed | Newline::NextLine;
    /// assert!(nlset1.is_superset(nlset2));
    /// assert!(!nlset1.is_superset(nlset3));
    /// ```
    pub fn is_superset(&self, other: NewlineSet) -> bool {
        other.is_subset(*self)
    }

    /// Returns an iterator over all [`Newline`] variants in `self` and/or
    /// `other`, without duplicates, in ascending order
    pub fn union(self, other: NewlineSet) -> Union {
        Union::new(self, other)
    }

    /// Returns an iterator over all [`Newline`] variants in both `self` and
    /// `other`, without duplicates, in ascending order
    pub fn intersection(self, other: NewlineSet) -> Intersection {
        Intersection::new(self, other)
    }

    /// Returns an iterator over all [`Newline`] variants in `self` or `other`
    /// but not both, without duplicates, in ascending order
    pub fn symmetric_difference(self, other: NewlineSet) -> SymmetricDifference {
        SymmetricDifference::new(self, other)
    }

    /// Returns an iterator over all [`Newline`] variants in `self` but not
    /// `other`, without duplicates, in ascending order
    pub fn difference(self, other: NewlineSet) -> Difference {
        Difference::new(self, other)
    }

    /// Returns an iterator over all [`Newline`] variants not in `self`,
    /// without duplicates, in ascending order
    pub fn complement(self) -> Complement {
        Complement::new(self)
    }

    /// Returns an iterator over all [`Newline`] variants in `self`, without
    /// duplicates, in ascending order
    pub fn iter(&self) -> IntoIter {
        self.into_iter()
    }
}

impl Ord for NewlineSet {
    // Same ordering logic as BTreeSet
    fn cmp(&self, other: &NewlineSet) -> core::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl PartialOrd for NewlineSet {
    fn partial_cmp(&self, other: &NewlineSet) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
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

impl ops::Not for NewlineSet {
    type Output = NewlineSet;

    fn not(self) -> NewlineSet {
        self.complement().into_newline_set()
    }
}

impl ops::Not for Newline {
    type Output = NewlineSet;

    fn not(self) -> NewlineSet {
        let mut nlset = NewlineSet::ALL;
        nlset.remove(self);
        nlset
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
            [core::cmp::min(nl1, nl2), core::cmp::max(nl1, nl2)]
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
            let [nl1, nl2]: [Newline; 2] = nls.try_into().unwrap();
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
            let [nl1, nl2]: [Newline; 2] = nls.try_into().unwrap();
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
                let [nl1, nl2]: [Newline; 2] = nls.try_into().unwrap();
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
            let nlset = NewlineSet::from_iter(core::iter::empty());
            assert_empty(nlset);
        }

        #[test]
        fn singleton() {
            for nl in Newline::iter() {
                let nlset = NewlineSet::from_iter(core::iter::once(nl));
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
            nlset.extend(core::iter::empty());
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

    #[rstest]
    #[case(Vec::new(), Newline::iter().collect())]
    #[case(Newline::iter().collect(), Vec::new())]
    #[case(
        vec![Newline::CarriageReturn, Newline::CrLf],
        vec![
            Newline::LineFeed,
            Newline::VerticalTab,
            Newline::FormFeed,
            Newline::NextLine,
            Newline::LineSeparator,
            Newline::ParagraphSeparator,
        ],
    )]
    #[case(
        vec![Newline::CarriageReturn, Newline::VerticalTab, Newline::FormFeed],
        vec![
            Newline::LineFeed,
            Newline::CrLf,
            Newline::NextLine,
            Newline::LineSeparator,
            Newline::ParagraphSeparator,
        ],
    )]
    fn test_complement(#[case] nlset: Vec<Newline>, #[case] comp: Vec<Newline>) {
        let nlset = NewlineSet::from_iter(nlset);
        assert_eq!(nlset.complement().collect_vec(), comp);
        let comp = NewlineSet::from_iter(comp);
        assert_eq!(!nlset, comp);
    }

    #[test]
    fn not_newline() {
        for nl in Newline::iter() {
            let not = !nl;
            for nl2 in Newline::iter() {
                assert_eq!(not.contains(nl2), nl != nl2);
            }
        }
    }
}
