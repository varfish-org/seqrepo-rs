//! Sequence repositories that operate on caches.
//!
//! This is useful in CI situations.  Locally, users can set environment variables
//! to have their tests use the cache writing implementation.  In the CI, they can
//! then use the cache reading implementation.

use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::error::Error;
use crate::repo::{self, AliasOrSeqId, SeqRepo};

/// Sequence repository reading from actual implementation and writing to a cache.
pub struct CacheWritingSeqRepo {
    /// Path to the cache file to write to.
    writer: Arc<Mutex<noodles_fasta::Writer<BufWriter<File>>>>,
    /// The actual implementation used for reading.
    repo: SeqRepo,
    /// The internal cache built when writing.
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl CacheWritingSeqRepo {
    pub fn new<P>(repo: SeqRepo, cache_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let cache = if cache_path.as_ref().exists() {
            Arc::new(Mutex::new(CacheReadingSeqRepo::read_cache(
                cache_path.as_ref(),
            )?))
        } else {
            Arc::new(Mutex::new(HashMap::new()))
        };
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&cache_path)
            .map_err(|e| Error::SeqSepoCacheOpenWrite(e.to_string()))?;
        Ok(Self {
            repo,
            writer: Arc::new(Mutex::new(noodles_fasta::Writer::new(BufWriter::new(file)))),
            cache,
        })
    }
}

impl repo::Interface for CacheWritingSeqRepo {
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, Error> {
        let key = build_key(alias_or_seq_id, begin, end);
        if let Some(value) = self
            .cache
            .as_ref()
            .lock()
            .expect("could not acquire lock")
            .get(&key)
        {
            return Ok(value.to_owned());
        }

        let value = self.repo.fetch_sequence_part(alias_or_seq_id, begin, end)?;
        self.cache
            .as_ref()
            .lock()
            .expect("could not acquire lock")
            .insert(key.clone(), value.clone());
        self.writer
            .lock()
            .expect("could not acquire lock")
            .write_record(&noodles_fasta::Record::new(
                noodles_fasta::record::Definition::new(key, None),
                noodles_fasta::record::Sequence::from(value.as_bytes().to_vec()),
            ))
            .map_err(|e| Error::SeqSepoCacheWrite(e.to_string()))?;
        Ok(value)
    }
}

/// Sequence repository reading from a cache.
pub struct CacheReadingSeqRepo {
    /// Map of query key to sequence.
    cache: HashMap<String, String>,
}

impl CacheReadingSeqRepo {
    pub fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            cache: Self::read_cache(path.as_ref())?,
        })
    }

    fn read_cache(path: &Path) -> Result<HashMap<String, String>, Error> {
        let mut reader = File::open(path)
            .map(BufReader::new)
            .map(noodles_fasta::Reader::new)
            .map_err(|e| Error::SeqSepoCacheOpenRead(e.to_string()))?;

        let mut result = HashMap::new();
        for record in reader.records() {
            let record = record.map_err(|e| Error::SeqSepoCacheRead(e.to_string()))?;
            result.insert(
                record.name().to_string(),
                std::str::from_utf8(record.sequence().as_ref())
                    .map_err(|e| Error::SeqSepoCacheOpenRead(e.to_string()))?
                    .to_string(),
            );
        }
        Ok(result)
    }
}

impl repo::Interface for CacheReadingSeqRepo {
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, Error> {
        let key = build_key(alias_or_seq_id, begin, end);
        if let Some(seq) = self.cache.get(&key) {
            Ok(seq.clone())
        } else {
            Err(Error::SeqSepoCacheKey(key))
        }
    }
}

fn build_key(alias_or_seq_id: &AliasOrSeqId, begin: Option<usize>, end: Option<usize>) -> String {
    let name = match alias_or_seq_id {
        AliasOrSeqId::Alias { value, namespace } => match namespace {
            Some(namespace) => {
                if namespace.is_empty() {
                    value.clone()
                } else {
                    format!("{namespace}:{value}")
                }
            }
            None => value.clone(),
        },
        AliasOrSeqId::SeqId(seqid) => seqid.clone(),
    };

    let suffix = match (begin, end) {
        (None, None) => "".to_string(),
        (None, Some(end)) => format!("?-{end}"),
        (Some(begin), None) => format!("{begin}-?"),
        (Some(begin), Some(end)) => format!("{begin}-{end}"),
    };

    if suffix.is_empty() {
        name
    } else {
        format!("{}:{}", &name, &suffix)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Error;
    use test_log::test;

    use std::{fs::read_to_string, path::PathBuf};

    use pretty_assertions::assert_eq;
    use temp_testdir::TempDir;

    use crate::{AliasOrSeqId, CacheReadingSeqRepo, Interface, SeqRepo};

    use super::CacheWritingSeqRepo;

    #[test]
    fn test_sync() {
        fn is_sync<T: Sync>() {}
        is_sync::<super::CacheReadingSeqRepo>();
        is_sync::<super::CacheWritingSeqRepo>();
    }

    fn test_fetch(sr: &impl Interface) -> Result<(), Error> {
        let alias = "NM_001304430.2";
        let aos = AliasOrSeqId::Alias {
            value: alias.to_string(),
            namespace: None,
        };

        assert_eq!(
            sr.fetch_sequence(&aos)?,
            "ACTGCTGAGCTGGGAGATGTCGGCGGCGTGTTGGGAGGAACCGTGGGGTCTTCCCGGCGGCTTT\
            GCGAAGCGGGTCCTGGTGACCGGCGGTGCTGGTTTCATGTAGGTAATGGCGCCGCTAGCCAAGCA\
            GTGGCTCCCCAGAAACCCCTACCTTTTCCCGCAGCTCTGCTTGCCCTAGTGCATCACATATGATT\
            GTCTCTTTAGTGGAAGATTATCCAAACTATATGATCATAAATCTAGACAAGCTGGATTACTGTGC\
            AAGCTTGAAGAATCTTGAAACCATTTCTAACAAACAGAACTACAAATTTATACAGGGTGACATAT\
            GTGATTCTCACTTTGTGAAACTGCTTTTTGAAACAGAGAAAATAGATATAGTACTACATTTTGCC\
            GCACAAACACATGTAGATCTTTCATTCGTACGTGCCTTTGAGTTTACCTATGTTAATGTTTATGG\
            CACTCACGTTTTGGTAAGTGCTGCTCATGAAGCCAGAGTGGAGAAGTTTATTTATGTCAGCACAG\
            ATGAAGTATATGGTGGCAGTCTTGATAAGGAATTTGATGAATCTTCACCCAAACAACCTACAAAT\
            CCTTATGCATCATCTAAAGCAGCTGCTGAATGTTTTGTACAGTCTTACTGGGAACAATATAAGTT\
            TCCAGTTGTCATCACAAGAAGCAGTAATGTTTATGGACCACATCAATATCCAGAAAAGGTTATTC\
            CAAAATTTATATCTTTGCTACAGCACAACAGGAAATGTTGCATTCATGGGTCAGGGCTTCAAACA\
            AGAAACTTCCTTTATGCTACTGATGTTGTAGAAGCATTTCTCACTGTCCTCAAAAAAGGGAAACC\
            AGGTGAAATTTATAACATCGGAACCAATTTTGAAATGTCAGTTGTCCAGCTTGCCAAAGAACTAA\
            TACAACTGATCAAAGAGACCAATTCAGAGTCTGAAATGGAAAATTGGGTTGATTATGTTAATGAT\
            AGACCCACCAATGACATGAGATACCCAATGAAGTCAGAAAAAATACATGGCTTAGGATGGAGACC\
            TAAAGTGCCTTGGAAAGAAGGAATAAAGAAAACAATTGAATGGTACAGAGAGAATTTTCACAACT\
            GGAAGAATGTGGAAAAGGCATTAGAACCCTTTCCGGTATAATCACCATTTATATAGTCGAGACAG\
            TTGTCAAAGAAGAAAGTTATCCTACCTCGCCAAGTGGTATGAAATTAAGTGACCAAATGAAGTGC\
            ACTCTTTTCTTTTGGAATTAGATTCATGACTTTCTGTATAAAATTCAAATGCAGAATGCCTCAAT\
            CTTTGGGAGAGTTTCAGTACTGGCATAGAATTTAAATGTCAAAATTCTTTCTGAAACCCTTTCTC\
            CTAGAAACTAGGAAATAATAGGTGTAGAAGACTCTCCCTAAGGGTAGCCAGGAAGAAGTCTCCTG\
            ATTCGGACAACCATGAGGGGTAGTGGTGCTAGGGAGAAGGCAACCTTCACTGGTTTTGAACTCAG\
            TGCCTAAGAAAGTCTCTGAAATGTTCGTTTTTAGGCAATATAGGATGTCTTAGGCCCTAATTCAC\
            CATTTCTTTTTTAAGATCTGATATGCTATCATTGCCTTAATAATGGAACAAAATAGAAGCATATC\
            TAACACTTTTTAAATTGATAATTTTGTAAAATTGATTACGTTGAATGCTTTTTAAGAGAAGTGTG\
            TAAAGTTTTTATATTTTCACAATTAACGTATGTAAAACCTTGTATCAGAAATTTATCATGTTTAC\
            TGTTTAAAATGATTGTATTTATAAAATTGTCAATATCTTAATGTATTTAATGTAGAATATTGCTT\
            TTTAAAATAATGTTTTTATTTTGCTGTAGAAAAATAAAAAAAAATTTGATTATA"
        );
        assert_eq!(sr.fetch_sequence_part(&aos, None, Some(4))?, "ACTG");
        assert_eq!(sr.fetch_sequence_part(&aos, Some(1869), None)?, "TATA");
        assert_eq!(sr.fetch_sequence_part(&aos, Some(0), Some(4))?, "ACTG");

        Ok(())
    }

    #[test]
    fn cache_writing() -> Result<(), Error> {
        let temp = TempDir::default();

        let sr = SeqRepo::new("tests/data/seqrepo", "latest")?;
        let mut cache_path = PathBuf::from(temp.as_ref());
        cache_path.push("cache.fasta");

        {
            let cw = CacheWritingSeqRepo::new(sr, &cache_path)?;
            test_fetch(&cw)?;
            // fetching twice will not change cache content
            test_fetch(&cw)?;
        }

        assert_eq!(
            read_to_string(&cache_path).unwrap(),
            read_to_string("tests/data/cached/cache.fasta").unwrap(),
        );

        Ok(())
    }

    #[test]
    fn cache_reading() -> Result<(), Error> {
        let cr = CacheReadingSeqRepo::new("tests/data/cached/cache.fasta")?;
        test_fetch(&cr)?;

        Ok(())
    }
}

// <LICENSE>
// Copyright 2023 seqrepo-rs Contributors
// Copyright 2016 biocommons.seqrepo Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// </LICENSE>
