//! This is the `seqrepo` library.

#[cfg(feature = "impl")]
pub(crate) mod aliases;
#[cfg(feature = "cached")]
pub(crate) mod cached;
pub(crate) mod error;
#[cfg(feature = "impl")]
pub(crate) mod fasta;
#[cfg(feature = "impl")]
pub(crate) mod interface;
#[cfg(feature = "impl")]
pub(crate) mod repo;

pub use crate::aliases::*;
#[cfg(feature = "cached")]
pub use crate::cached::*;
pub use crate::error::*;
#[cfg(feature = "impl")]
pub use crate::fasta::*;
#[cfg(feature = "impl")]
pub use crate::interface::*;
#[cfg(feature = "impl")]
pub use crate::repo::*;
