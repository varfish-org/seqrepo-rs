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
    path::Path,
    rc::Rc,
};

use noodles::fasta;

use crate::repo::{AliasOrSeqId, Interface as SeqRepoInterface, SeqRepo};

/// Sequence repository reading from actual implementation and writing to a cache.
pub struct CacheWritingSeqRepo {
    /// Path to the cache file to write to.
    writer: Rc<RefCell<fasta::Writer<BufWriter<File>>>>,
    /// The actual implementation used for reading.
    repo: SeqRepo,
    /// The internal cache built when writing.
    cache: Rc<RefCell<HashMap<String, String>>>,
}

impl CacheWritingSeqRepo {
    pub fn new(repo: SeqRepo, cache: File) -> Self {
        Self {
            repo,
            writer: Rc::new(RefCell::new(fasta::Writer::new(BufWriter::new(cache)))),
            cache: Rc::new(RefCell::new(HashMap::new())),
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
        if let Some(value) = self.cache.as_ref().borrow().get(&key) {
            return Ok(value.to_owned());
        }

        let value = self.repo.fetch_sequence_part(alias_or_seq_id, begin, end)?;
        self.cache
            .as_ref()
            .borrow_mut()
            .insert(key.clone(), value.clone());
        self.writer.borrow_mut().write_record(&fasta::Record::new(
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
    pub fn new<P>(path: P) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        Ok(Self {
            cache: Self::read_cache(path.as_ref())?,
        })
    }

    fn read_cache(path: &Path) -> Result<HashMap<String, String>, anyhow::Error> {
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
    use std::{
        fs::{read_to_string, File},
        path::PathBuf,
    };

    use pretty_assertions::assert_eq;
    use temp_testdir::TempDir;

    use crate::{AliasOrSeqId, CacheReadingSeqRepo, Interface, SeqRepo};

    use super::CacheWritingSeqRepo;

    fn test_fetch(sr: &impl Interface) -> Result<(), anyhow::Error> {
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
    fn cache_writing() -> Result<(), anyhow::Error> {
        let temp = TempDir::default();

        let sr = SeqRepo::new("tests/data/seqrepo", "latest")?;
        let mut cache_path = PathBuf::from(temp.as_ref());
        cache_path.push("cache.fasta");
        let cache = File::create(cache_path.clone())?;

        {
            let cw = CacheWritingSeqRepo::new(sr, cache);
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
    fn cache_reading() -> Result<(), anyhow::Error> {
        let cr = CacheReadingSeqRepo::new("tests/data/cached/cache.fasta")?;
        test_fetch(&cr)?;

        Ok(())
    }
}

// <LICENSE>
// Copyright 2023 hgvs-rs Contributors
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
