//! Error type definition.

use thiserror::Error;

/// Error type for variant mapping.
#[derive(Error, Debug)]
pub enum Error {
    // #[error("validation error")]
    // ValidationFailed(#[from] crate::validator::Error),
    // ExpectedGenomeVariant(String),
    // #[error("expected a TxVariant but received {0}")]
    #[error("error on connecting to database: {0}")]
    AliasDbConnect(String),
    #[error("error on preparing statement: {0}")]
    AliasDbStmt(String),
    #[error("error executing query: {0}")]
    AliasDbExec(String),
    #[error("error on row: {0}")]
    AliasDbQuery(String),
    #[error("error opening cache file for writing: {0}")]
    SeqSepoCacheOpenWrite(String),
    #[error("error writing to cache file: {0}")]
    SeqSepoCacheWrite(String),
    #[error("error opening cache file for reading: {0}")]
    SeqSepoCacheOpenRead(String),
    #[error("error reading from cache file: {0}")]
    SeqSepoCacheRead(String),
    #[error("key not found in cache: {0}")]
    SeqSepoCacheKey(String),
    #[error("upgrade required: database schema version is {0} and the code expects {1}")]
    SeqSepoDbSchemaVersion(u32, u32),
    #[error("error on connecting to database: {0}")]
    SeqRepoDbConnect(String),
    #[error("error on preparing statement: {0}")]
    SeqRepoDbStmt(String),
    #[error("error executing query: {0}")]
    SeqRepoDbExec(String),
    #[error("error on row: {0}")]
    SeqRepoDbQuery(String),
    #[error("error opening FAI index: {0}")]
    SeqRepoFaiOpen(String),
    #[error("error opening BGZF reader: {0}")]
    SeqRepoBgzfOpen(String),
    #[error("error opening GZI index: {0}")]
    SeqRepoGziOpen(String),
    #[error("error opening FASTA reader: {0}")]
    SeqRepoFastaOpen(String),
    #[error("error converting position: {0}")]
    ConvertPosition(String),
    #[error("error querying FAI reader: {0}")]
    SeqRepoFaiQuery(String),
    #[error("could not resolve alias {0} to seqid")]
    AliasDbResolve(String),
    #[error("alias {0} resolved to multiple seqids {1}")]
    AliasDbResolutionAmbiguous(String, String),
}
