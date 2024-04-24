use super::nl::{CharType, Newline};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NewlineSet {
    /// A super-array of the `&[char]` pattern used to search strings for the
    /// Newlines in the set.  `pattern_buf` consists of the `as_char()` of each
    /// Newline in the set in codepoint order, with trailing unused elements
    /// set to '\0'.
    ///
    /// If CrLf is in the set, then `pattern_buf` contains '\r'.  If both CrLf
    /// and CarriageReturn are in the set, `pattern_buf` will only contain one
    /// '\r'.
    pattern_buf: [char; Newline::COUNT],

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
        NewlineSet {
            pattern_buf: ['\0'; Newline::COUNT],
            pattern_len: 0,
            cr: false,
            crlf: false,
        }
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

    pub fn insert(&mut self, nl: Newline) {
        let ch = match nl.chartype() {
            CharType::Char(ch) => ch,
            CharType::CrLf => '\r',
        };
        if let Err(i) = self.pattern_buf[..self.pattern_len].binary_search(&ch) {
            self.pattern_buf[i..].rotate_right(1);
            self.pattern_buf[i] = ch;
            self.pattern_len += 1;
        }
        if nl == Newline::CarriageReturn {
            self.cr = true;
        } else if nl == Newline::CrLf {
            self.crlf = true;
        }
    }
}

impl Default for NewlineSet {
    fn default() -> NewlineSet {
        NewlineSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let nlset = NewlineSet::new();
        assert_eq!(nlset.len(), 0);
        assert!(nlset.is_empty());
        for nl in Newline::iter() {
            assert!(!nlset.contains(nl));
        }
    }

    #[test]
    fn test_insert_one() {
        for nl in Newline::iter() {
            let mut nlset = NewlineSet::new();
            nlset.insert(nl);
            assert_eq!(nlset.len(), 1);
            assert!(!nlset.is_empty());
            for nl2 in Newline::iter() {
                assert_eq!(nlset.contains(nl2), nl == nl2);
            }
        }
    }
}
