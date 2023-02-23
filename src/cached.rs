//! Sequence repositories that operate on caches.
//!
//! This is useful in CI situations.  Locally, users can set environment variables
//! to have their tests use the cache writing implementation.  In the CI, they can
//! then use the cache reading implementation.

use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    rc::Rc,
};

use noodles::fasta;

use crate::repo::{AliasOrSeqId, Interface as SeqRepoInterface, SeqRepo};

/// Sequence repository reading from actual implementation and writing to a cache.
pub struct CacheWritingSeqRepo {
    /// Path to the cache file to write to.
    cache: Rc<RefCell<fasta::Writer<BufWriter<File>>>>,
    /// The actual implementation used for reading.
    repo: SeqRepo,
}

impl CacheWritingSeqRepo {
    pub fn new(repo: SeqRepo, cache: File) -> Self {
        Self {
            repo,
            cache: Rc::new(RefCell::new(fasta::Writer::new(BufWriter::new(cache)))),
        }
    }
}

impl SeqRepoInterface for CacheWritingSeqRepo {
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, anyhow::Error> {
        let key = build_key(alias_or_seq_id, begin, end);
        let value = self.repo.fetch_sequence_part(alias_or_seq_id, begin, end)?;
        self.cache.borrow_mut().write_record(&fasta::Record::new(
            fasta::record::Definition::new(key, None),
            fasta::record::Sequence::from(value.as_bytes().to_vec()),
        ))?;
        Ok(value)
    }
}

/// Sequence repository reading from a cache.
pub struct CacheReadingSeqRepo {
    /// Map of query key to sequence.
    cache: HashMap<String, String>,
}

impl CacheReadingSeqRepo {
    pub fn new(path: &PathBuf) -> Result<Self, anyhow::Error> {
        Ok(Self {
            cache: Self::read_cache(path)?,
        })
    }

    fn read_cache(path: &PathBuf) -> Result<HashMap<String, String>, anyhow::Error> {
        let mut reader = File::open(path)
            .map(BufReader::new)
            .map(fasta::Reader::new)?;

        let mut result = HashMap::new();
        for record in reader.records() {
            let record = record?;
            result.insert(
                record.name().to_string(),
                std::str::from_utf8(record.sequence().as_ref())?.to_string(),
            );
        }
        Ok(result)
    }
}

impl SeqRepoInterface for CacheReadingSeqRepo {
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, anyhow::Error> {
        let key = build_key(alias_or_seq_id, begin, end);
        if let Some(seq) = self.cache.get(&key) {
            Ok(seq.clone())
        } else {
            Err(anyhow::anyhow!("Key {} not found in cache.", &key))
        }
    }
}

fn build_key(alias_or_seq_id: &AliasOrSeqId, begin: Option<usize>, end: Option<usize>) -> String {
    let name = match alias_or_seq_id {
        AliasOrSeqId::Alias { value, namespace } => match namespace {
            Some(namespace) => format!("{}:{}", namespace, value),
            None => value.clone(),
        },
        AliasOrSeqId::SeqId(seqid) => seqid.clone(),
    };

    let suffix = match (begin, end) {
        (None, None) => "".to_string(),
        (None, Some(end)) => format!("?-{}", end),
        (Some(begin), None) => format!("{}-?", begin),
        (Some(begin), Some(end)) => format!("{}-{}", begin, end),
    };

    format!("{}:{}", &name, &suffix)
}
