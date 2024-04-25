use super::nl::Newline;
use std::cmp::Ordering;
use std::iter::FusedIterator;

/// A set of at most `Newline::COUNT - 1` characters.
///
/// The maximum size is the number of distinct initial characters of the string
/// representations of the `Newline` variants.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) struct CharSet {
    /// The first `len` elements of `data` are the elements of the `CharSet`,
    /// stored in strictly ascending order.  Any remaining elements are set to
    /// `'\0'`.
    data: [char; Newline::COUNT - 1],

    /// The number of items in the `CharSet`.
    len: usize,
}

impl CharSet {
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub(crate) fn as_slice(&self) -> &[char] {
        &self.data[..self.len]
    }

    pub(crate) fn get(&self, i: usize) -> Option<char> {
        (i < self.len).then(|| self.data[i])
    }

    /// Adds `ch` to the set.  Returns `true` if `ch` was not already in the
    /// set.
    ///
    /// # Panics
    ///
    /// Panics if the `CharSet` is full.
    pub(crate) fn insert(&mut self, ch: char) -> bool {
        match self.as_slice().binary_search(&ch) {
            Ok(_) => false,
            Err(i) => {
                assert!(i < self.data.len(), "Attempted insert on full CharSet");
                self.data[i..].rotate_right(1);
                self.data[i] = ch;
                self.len += 1;
                true
            }
        }
    }

    /// Removes `ch` from the set, if present.  Returns `true` if `ch` was in
    /// the set.
    pub(crate) fn remove(&mut self, ch: char) -> bool {
        match self.as_slice().binary_search(&ch) {
            Ok(i) => {
                debug_assert!(i < self.data.len(), "i should be less than data len");
                self.data[i] = '\0';
                self.data[i..].rotate_left(1);
                self.len -= 1;
                true
            }
            Err(_) => false,
        }
    }

    pub(crate) fn contains(&self, ch: char) -> bool {
        self.as_slice().binary_search(&ch).is_ok()
    }

    pub(crate) fn append(&mut self, ch: char) {
        self.data[self.len] = ch;
        self.len += 1;
    }

    pub(crate) fn diff(self, other: CharSet) -> CharSetDiff {
        CharSetDiff::new(self, other)
    }
}

impl IntoIterator for CharSet {
    type Item = char;
    type IntoIter = CharSetIter;

    fn into_iter(self) -> CharSetIter {
        CharSetIter::new(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CharSetIter {
    charset: CharSet,
    i: usize,
}

impl CharSetIter {
    pub(crate) fn new(charset: CharSet) -> CharSetIter {
        CharSetIter { charset, i: 0 }
    }

    pub(crate) fn peek(&self) -> Option<char> {
        self.charset.get(self.i)
    }

    pub(crate) fn peek_back(&self) -> Option<char> {
        self.charset
            .len()
            .checked_sub(1)
            .filter(|&j| j >= self.i)
            .map(|j| self.charset.data[j])
    }
}

impl Iterator for CharSetIter {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        let r = self.charset.get(self.i);
        if r.is_some() {
            self.i += 1;
        }
        r
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.charset.len() - self.i;
        (sz, Some(sz))
    }
}

impl FusedIterator for CharSetIter {}

impl ExactSizeIterator for CharSetIter {}

impl DoubleEndedIterator for CharSetIter {
    fn next_back(&mut self) -> Option<char> {
        let r = self.peek_back();
        if r.is_some() {
            self.charset.len -= 1;
        }
        r
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Diff {
    Left(char),
    Both(char),
    Right(char),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CharSetDiff {
    left_iter: CharSetIter,
    right_iter: CharSetIter,
}

impl CharSetDiff {
    pub(crate) fn new(left: CharSet, right: CharSet) -> CharSetDiff {
        CharSetDiff {
            left_iter: left.into_iter(),
            right_iter: right.into_iter(),
        }
    }
}

impl Iterator for CharSetDiff {
    type Item = Diff;

    fn next(&mut self) -> Option<Diff> {
        match (self.left_iter.peek(), self.right_iter.peek()) {
            (Some(lc), Some(rc)) => match lc.cmp(&rc) {
                Ordering::Less => {
                    self.left_iter.next();
                    Some(Diff::Left(lc))
                }
                Ordering::Equal => {
                    self.left_iter.next();
                    self.right_iter.next();
                    Some(Diff::Both(lc))
                }
                Ordering::Greater => {
                    self.right_iter.next();
                    Some(Diff::Right(rc))
                }
            },
            (Some(ch), None) => {
                self.left_iter.next();
                Some(Diff::Left(ch))
            }
            (None, Some(ch)) => {
                self.right_iter.next();
                Some(Diff::Right(ch))
            }
            (None, None) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    mod diff {
        use super::*;

        #[test]
        fn empty_empty() {
            let cs1 = CharSet::default();
            let cs2 = CharSet::default();
            assert_eq!(cs1.diff(cs2).count(), 0);
        }

        #[test]
        fn single_empty() {
            let mut cs1 = CharSet::default();
            cs1.insert('a');
            let cs2 = CharSet::default();
            assert_eq!(cs1.diff(cs2).collect_vec(), [Diff::Left('a')]);
        }

        #[test]
        fn empty_single() {
            let cs1 = CharSet::default();
            let mut cs2 = CharSet::default();
            cs2.insert('a');
            assert_eq!(cs1.diff(cs2).collect_vec(), [Diff::Right('a')]);
        }

        #[test]
        fn equal_single() {
            let mut cs1 = CharSet::default();
            cs1.insert('a');
            let mut cs2 = CharSet::default();
            cs2.insert('a');
            assert_eq!(cs1.diff(cs2).collect_vec(), [Diff::Both('a')]);
        }

        #[test]
        fn single_lt_single() {
            let mut cs1 = CharSet::default();
            cs1.insert('a');
            let mut cs2 = CharSet::default();
            cs2.insert('b');
            assert_eq!(
                cs1.diff(cs2).collect_vec(),
                [Diff::Left('a'), Diff::Right('b')]
            );
        }

        #[test]
        fn single_gt_single() {
            let mut cs1 = CharSet::default();
            cs1.insert('b');
            let mut cs2 = CharSet::default();
            cs2.insert('a');
            assert_eq!(
                cs1.diff(cs2).collect_vec(),
                [Diff::Right('a'), Diff::Left('b')]
            );
        }

        #[test]
        fn misc01() {
            let mut cs1 = CharSet::default();
            cs1.insert('a');
            cs1.insert('c');
            cs1.insert('e');
            let mut cs2 = CharSet::default();
            cs2.insert('b');
            cs2.insert('c');
            cs2.insert('d');
            assert_eq!(
                cs1.diff(cs2).collect_vec(),
                [
                    Diff::Left('a'),
                    Diff::Right('b'),
                    Diff::Both('c'),
                    Diff::Right('d'),
                    Diff::Left('e'),
                ]
            );
        }
    }
}
