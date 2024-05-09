//! Error types
use std::fmt;

/// Error returned by `TryFrom<char> for Newline` when given a `char` that is
/// not a recognized newline sequence
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TryFromCharError(
    /// The character provided to `TryFrom`
    pub char,
);

impl fmt::Display for TryFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not a newline character", self.0)
    }
}

//#[cfg(feature = "std")]
//#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for TryFromCharError {}

/// Error returned by `TryFrom<&str> for Newline` when given a `&str` that is
/// not a recognized newline sequence
#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TryFromStrError;

impl fmt::Display for TryFromStrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("string is not a newline sequence")
    }
}

//#[cfg(feature = "std")]
//#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for TryFromStrError {}
