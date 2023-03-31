//! Code providing the interface for sequence repositories and the base `SeqRepo` implementation.

use std::path::{Path, PathBuf};

use crate::{AliasDb, FastaDir, Namespace, Query};

/// Trait describing the interface of a sequence repository.
pub trait Interface {
    /// Fetch part sequence given an alias.
    fn fetch_sequence(&self, alias_or_seq_id: &AliasOrSeqId) -> Result<String, anyhow::Error> {
        self.fetch_sequence_part(alias_or_seq_id, None, None)
    }

    /// Fetch part sequence given an alias.
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, anyhow::Error>;
}

/// Provide (read-only) access to a `seqrepo` sequence repository.
#[derive(Debug)]
pub struct SeqRepo {
    /// The path to the seqrepo root directory.
    root_dir: PathBuf,
    /// The name of the instance.
    instance: String,
    /// The `AliasDb` to use.
    alias_db: AliasDb,
    /// The `FastaDir` to use.
    fasta_dir: FastaDir,
}

pub enum AliasOrSeqId {
    Alias {
        value: String,
        namespace: Option<String>,
    },
    SeqId(String),
}

impl SeqRepo {
    /// Create new `SeqRepo` at the given path.
    pub fn new<P>(path: P, instance: &str) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let root_dir = PathBuf::from(path.as_ref());
        let instance = instance.to_string();
        let alias_db = AliasDb::new(&root_dir, &instance)?;
        let path_fasta_dir = root_dir.join(&instance).join("sequences");
        let fasta_dir = FastaDir::new(path_fasta_dir)?;
        Ok(SeqRepo {
            root_dir,
            instance,
            alias_db,
            fasta_dir,
        })
    }

    /// Provide access to the root directory.
    pub fn root_dir(&self) -> &Path {
        self.root_dir.as_ref()
    }

    /// Provide access to the instance name.
    pub fn instance(&self) -> &str {
        &self.instance
    }

    /// Provide access to `AliasDb`.
    pub fn alias_db(&self) -> &AliasDb {
        &self.alias_db
    }

    /// Provide access to `FastaDir`.
    pub fn fasta_dir(&self) -> &FastaDir {
        &self.fasta_dir
    }
}

impl Interface for SeqRepo {
    fn fetch_sequence_part(
        &self,
        alias_or_seq_id: &AliasOrSeqId,
        begin: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, anyhow::Error> {
        let seq_ids = match alias_or_seq_id {
            AliasOrSeqId::Alias { value, namespace } => {
                let query = Query {
                    namespace: namespace.as_ref().map(|s| Namespace::new(s)),
                    alias: Some(value.to_string()),
                    ..Default::default()
                };
                let mut seq_ids = Vec::new();
                self.alias_db
                    .find(&query, |record| seq_ids.push(record.unwrap().seqid))?;

                if seq_ids.is_empty() {
                    return Err(anyhow::anyhow!(
                        "Could not resolve alias {} to seqid!",
                        value
                    ));
                } else if seq_ids.len() > 1 {
                    return Err(anyhow::anyhow!(
                        "Alias {} resolved to multiple seqids: {:?}",
                        value,
                        &seq_ids
                    ));
                }

                seq_ids
            }
            AliasOrSeqId::SeqId(seqid) => vec![seqid.clone()],
        };

        self.fasta_dir.fetch_sequence_part(&seq_ids[0], begin, end)
    }
}

#[cfg(test)]
mod test {
    use crate::{repo::Interface, AliasOrSeqId, SeqRepo};

    #[test]
    fn seqrepo_smoke() -> Result<(), anyhow::Error> {
        let sr = SeqRepo::new("tests/data/seqrepo", "latest")?;
        assert_eq!(
            sr.root_dir().to_str().unwrap().to_string(),
            "tests/data/seqrepo".to_string(),
        );
        assert_eq!(sr.instance(), "latest".to_string(),);
        sr.alias_db();
        sr.fasta_dir();

        Ok(())
    }

    #[test]
    fn fetch_sequence() -> Result<(), anyhow::Error> {
        let sr = SeqRepo::new("tests/data/seqrepo", "latest")?;
        let alias = "NM_001304430.2";

        assert_eq!(
            sr.fetch_sequence(&AliasOrSeqId::Alias {
                value: alias.to_string(),
                namespace: None
            })?,
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

        Ok(())
    }

    #[test]
    fn fetch_sequence_part() -> Result<(), anyhow::Error> {
        let sr = SeqRepo::new("tests/data/seqrepo", "latest")?;
        let alias = "NM_001304430.2";

        assert_eq!(
            sr.fetch_sequence_part(
                &AliasOrSeqId::Alias {
                    value: alias.to_string(),
                    namespace: None
                },
                Some(0),
                Some(10)
            )?,
            "ACTGCTGAGC"
        );
        assert_eq!(
            sr.fetch_sequence_part(
                &AliasOrSeqId::Alias {
                    value: alias.to_string(),
                    namespace: None
                },
                Some(100),
                Some(110)
            )?,
            "ATGTAGGTAA"
        );

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
