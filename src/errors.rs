use std::fmt;

#[derive(Copy, Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TryFromCharError(pub char);

impl fmt::Display for TryFromCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} is not a newline character", self.0)
    }
}

//#[cfg(feature = "std")]
//#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
impl std::error::Error for TryFromCharError {}

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
