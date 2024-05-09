#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
mod charset;
pub mod errors;
pub mod iter;
mod nl;
mod nlset;
mod pattern;
pub use self::nl::*;
pub use self::nlset::*;
pub use self::pattern::*;
