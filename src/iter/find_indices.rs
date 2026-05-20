use crate::pattern::NewlinePattern;
use core::iter::FusedIterator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FindIndices<'a, NP> {
    pattern: &'a NP,
    s: &'a str,
    offset: usize,
}

impl<'a, NP> FindIndices<'a, NP> {
    pub(crate) fn new(pattern: &'a NP, s: &'a str) -> Self {
        FindIndices {
            pattern,
            s,
            offset: 0,
        }
    }
}

impl<NP: NewlinePattern> Iterator for FindIndices<'_, NP> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.s.is_empty() {
            return None;
        }
        let Some((start, end)) = self.pattern.search(self.s) else {
            self.s = "";
            return None;
        };
        self.s = &self.s[end..];
        let start = start.saturating_add(self.offset);
        let end = end.saturating_add(self.offset);
        self.offset = end;
        Some((start, end))
    }
}

impl<NP: NewlinePattern> FusedIterator for FindIndices<'_, NP> {}

impl<NP: NewlinePattern> DoubleEndedIterator for FindIndices<'_, NP> {
    fn next_back(&mut self) -> Option<(usize, usize)> {
        if self.s.is_empty() {
            return None;
        }
        let Some((start, end)) = self.pattern.rsearch(self.s) else {
            self.s = "";
            return None;
        };
        self.s = &self.s[..start];
        let start = start.saturating_add(self.offset);
        let end = end.saturating_add(self.offset);
        Some((start, end))
    }
}
