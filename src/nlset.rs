use super::nl::{CharType, Newline};

#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct NewlineSet {
    /// A super-array of the `&[char]` pattern used to search strings for the
    /// Newlines in the set.  `pattern_buf` consists of the `as_char()` of each
    /// Newline in the set in codepoint order, with trailing unused elements
    /// set to '\0'.
    ///
    /// If CrLf is in the set, then `pattern_buf` contains '\r'.  If both CrLf
    /// and CarriageReturn are in the set, `pattern_buf` will only contain one
    /// '\r'.  (Hence, the length of the array can be one less than
    /// `Newline::COUNT`.)
    pattern_buf: [char; Newline::COUNT - 1],

    /// The length of the pattern in `pattern_buf`, i.e., the number of leading
    /// non-NUL elements
    pattern_len: usize,

    /// Whether CarriageReturn is in the set
    cr: bool,

    /// Whether CrLf is in the set
    crlf: bool,
}

impl NewlineSet {
    pub fn new() -> NewlineSet {
        NewlineSet::default()
    }

    pub fn len(&self) -> usize {
        self.pattern_len + usize::from(self.cr && self.crlf)
    }

    pub fn is_empty(&self) -> bool {
        self.pattern_len == 0
    }

    pub fn contains(&self, nl: Newline) -> bool {
        match nl.chartype() {
            CharType::Char('\r') => self.cr,
            CharType::Char(ch) => self.pattern_buf[..self.pattern_len]
                .binary_search(&ch)
                .is_ok(),
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
        match self.pattern_buf[..self.pattern_len].binary_search(&ch) {
            Ok(_) => false,
            Err(i) => {
                self.pattern_buf[i..].rotate_right(1);
                self.pattern_buf[i] = ch;
                self.pattern_len += 1;
                true
            }
        }
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
        match self.pattern_buf[..self.pattern_len].binary_search(&ch) {
            Ok(i) => {
                self.pattern_buf[i] = '\0';
                self.pattern_buf[i..].rotate_left(1);
                self.pattern_len -= 1;
                true
            }
            Err(_) => false,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

impl From<Newline> for NewlineSet {
    fn from(nl: Newline) -> NewlineSet {
        let mut nlset = NewlineSet::new();
        nlset.insert(nl);
        nlset
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
        assert!(nlset.pattern_buf.into_iter().all(|ch| ch == '\0'));
        assert_eq!(nlset.pattern_len, 0);
        assert!(!nlset.cr);
        assert!(!nlset.crlf);
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
            assert_eq!(nlset.len(), 1);
            assert!(!nlset.is_empty());
            for nl2 in Newline::iter() {
                assert_eq!(nlset.contains(nl2), nl == nl2);
            }
        }
    }

    #[test]
    fn test_insert_one_twice() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            assert!(nlset.insert(nl));
            assert!(!nlset.insert(nl));
            assert_eq!(nlset.len(), 1);
            assert!(!nlset.is_empty());
            for nl2 in Newline::iter() {
                assert_eq!(nlset.contains(nl2), nl == nl2);
            }
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
            assert_eq!(nlset.len(), 2);
            assert!(!nlset.is_empty());
            for nl in Newline::iter() {
                assert_eq!(nlset.contains(nl), nl == nl1 || nl == nl2);
            }
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
            assert_eq!(nlset.len(), 1);
            assert!(!nlset.is_empty());
            for nl in Newline::iter() {
                assert_eq!(nlset.contains(nl), nl == nl2);
            }
        }
    }

    #[test]
    fn test_from_newline() {
        for nl in Newline::iter() {
            let nlset = NewlineSet::from(nl);
            assert_eq!(nlset.len(), 1);
            assert!(!nlset.is_empty());
            for nl2 in Newline::iter() {
                assert_eq!(nlset.contains(nl2), nl == nl2);
            }
        }
    }
}
