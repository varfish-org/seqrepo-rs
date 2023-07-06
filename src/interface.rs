//! Implementation of the interface trait.

use crate::error::Error;

/// Trait describing the interface of a sequence repository.
pub trait Interface {
    /// Fetch part sequence given an alias.
    fn fetch_sequence(&self, alias_or_seq_id: &AliasOrSeqId) -> Result<String, Error> {
        self.fetch_sequence_part(alias_or_seq_id, None, None)
    }

    /// Fetch part sequence given an alias.
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, Error>;
}

pub enum AliasOrSeqId {
    Alias {
        value: String,
        namespace: Option<String>,
    },
    SeqId(String),
}
