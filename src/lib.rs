//! Code for providing SeqRepo in Rust.

mod aliases;

use std::path::{Path, PathBuf};

use rusqlite::Result;

pub use crate::aliases::*;

/// Provide (read-only) access to a `seqrepo` sequence repository.
#[derive(Debug)]
pub struct SeqRepo {
    /// The path to the seqrepo root directory.
    root_dir: PathBuf,
    /// The name of the instance.
    instance: String,
}

impl SeqRepo {
    /// Create new `SeqRepo` at the given path.
    pub fn new<P>(path: P, instance: &str) -> Self
    where
        P: AsRef<Path>,
    {
        SeqRepo {
            root_dir: PathBuf::from(path.as_ref()),
            instance: instance.to_string(),
        }
    }

    /// Get access to aliases.
    pub fn aliases(&self) -> Result<Aliases, anyhow::Error> {
        Aliases::new(&self.root_dir, &self.instance)
    }

    /// Provide access to the root directory.
    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_ref()
    }

    /// Provide access to the instance name
    pub fn instance(&self) -> &str {
        &self.instance
    }
}

#[cfg(test)]
mod test {
    use crate::SeqRepo;

    #[test]
    fn seqrepo_create() {
        let sr = SeqRepo::new("tests/data/seqrepo", "latest");
        assert_eq!(
            sr.root_dir().to_str().unwrap().to_string(),
            "tests/data/seqrepo".to_string(),
        );
        assert_eq!(sr.instance(), "latest".to_string(),);
    }
}
