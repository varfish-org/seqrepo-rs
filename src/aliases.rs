//! Access to the aliases databas.

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use rusqlite::{types::Value, Connection, OpenFlags};
use tracing::{debug, trace};

/// Namespaces as stored in the database.
///
/// The string values returned by the `Display` trait are the values stored in
/// the database.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Namespace(pub String);

/// A pair of namespace and alias as available in the database.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NamespacedAlias {
    pub namespace: Namespace,
    pub alias: String,
}

/// Datastructure for a query to `Aliases::find()`.
#[derive(Debug)]
pub struct Query {
    /// Optionally, namespace to query within.
    pub namespace: Option<Namespace>,
    /// Optionally, an alias or pattern using `%` for wildcards.
    pub alias: Option<String>,
    /// Optionally the precise seqid.
    pub seqid: Option<String>,
    /// Whether to return those with `is_current=1`.
    pub current_only: bool,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            namespace: Default::default(),
            alias: Default::default(),
            seqid: Default::default(),
            current_only: true,
        }
    }
}

/// Record as returned by `Aliases::find()`.
#[derive(Debug)]
pub struct AliasRecord {
    pub seqalias_id: u64,
    pub seqid: String,
    pub alias: String,
    pub added: NaiveDateTime,
    pub is_current: bool,
    pub namespace: Namespace,
}

/// Provides access to the aliases database of the `SeqRepo`.
#[derive(Debug)]
pub struct Aliases {
    /// The path to the seqrepo root directory.
    sr_root_dir: PathBuf,
    /// The name of the seqrepo instance.
    sr_instance: String,
    /// Connection to the SQLite database.
    conn: Connection,
}

impl Aliases {
    pub fn new<P>(sr_root_dir: &P, sr_instance: &str) -> Result<Self, anyhow::Error>
    where
        P: AsRef<Path>,
    {
        let sr_root_dir = PathBuf::from(sr_root_dir.as_ref());
        let sr_instance = sr_instance.to_string();
        let conn = Self::new_connection(&sr_root_dir, &sr_instance)?;

        Ok(Aliases {
            sr_root_dir,
            sr_instance,
            conn,
        })
    }

    fn new_connection(sr_root_dir: &Path, sr_instance: &str) -> Result<Connection, anyhow::Error> {
        let db_path = sr_root_dir.join(&sr_instance).join("aliases.sqlite3");
        Ok(Connection::open_with_flags(
            db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?)
    }

    /// Try to clone the `Aliases`.
    ///
    /// A new sqlite connection must be made so this can fail.
    pub fn try_clone(&self) -> Result<Self, anyhow::Error> {
        Ok(Self {
            sr_root_dir: self.sr_root_dir.clone(),
            sr_instance: self.sr_instance.clone(),
            conn: Self::new_connection(&self.sr_root_dir, &self.sr_instance)?,
        })
    }

    /// Find aliases an call `f` on each result record.
    ///
    /// The arguments, all optional, restrict the records that are returned, possibly all.
    ///
    /// Regardless of the query, results are ordered by `seq_id`.
    ///
    /// If `query.alias` or `query.seqid` contain `%`, the `like` comparison operator is
    // used.  Otherwise arguments must match exactly.
    pub fn find<F>(&self, query: &Query, mut f: F) -> Result<(), anyhow::Error>
    where
        F: FnMut(Result<AliasRecord, anyhow::Error>),
    {
        trace!("Aliases::find({:?})", &query);
        fn eq_or_like(s: &str) -> &'static str {
            if s.contains("%") {
                "like"
            } else {
                "="
            }
        }

        let mut clauses = Vec::new();
        let mut params: Vec<rusqlite::types::Value> = Vec::new();

        // Add namespace to query if provided.
        if let Some(Namespace(namespace)) = &query.namespace {
            let namespace = format!("{}", &namespace);
            clauses.push(format!("namespace {} ?", eq_or_like(&namespace)));
            params.push(Value::Text(namespace));
        }
        // Add alias to query if provided.
        if let Some(alias) = query.alias.as_deref() {
            clauses.push(format!("alias {} ?", eq_or_like(alias)));
            params.push(Value::Text(alias.to_string()));
        }
        // Add seqid to query if provided.
        if let Some(seqid) = query.seqid.as_deref() {
            clauses.push(format!("alias {} ?", eq_or_like(seqid)));
            params.push(Value::Text(seqid.to_string()));
        }
        // Possibly limit to the current ones only.
        if query.current_only {
            clauses.push(format!("is_current = 1"));
        }

        // Prepare SQL query.
        let cols = &[
            "seqalias_id",
            "seq_id",
            "alias",
            "added",
            "is_current",
            "namespace",
        ];
        let mut sql = format!("SELECT {} FROM seqalias", &cols.join(", "));
        if !clauses.is_empty() {
            sql.push_str(" WHERE ");
            let clauses: Vec<_> = clauses.iter().map(|s| format!("({})", s)).collect();
            sql.push_str(&clauses.join(" AND "));
        }
        sql.push_str(" ORDER BY seq_id, namespace, alias");
        debug!("Executing: {:?} with params {:?}", &sql, &params);

        let mut stmt = self.conn.prepare(&sql)?;

        for row in stmt.query_map(rusqlite::params_from_iter(params), |row| {
            let added: String = row.get(3)?;
            let added = NaiveDateTime::parse_from_str(&added, "%Y-%m-%d %H:%M:%S")
                .expect("could not convert timestamp");
            Ok(AliasRecord {
                seqalias_id: row.get(0)?,
                seqid: row.get(1)?,
                alias: row.get(2)?,
                added,
                is_current: row.get(4)?,
                namespace: Namespace(row.get(5)?),
            })
        })? {
            f(row.map_err(|e| anyhow::anyhow!("Error on row: {}", &e)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use pretty_assertions::assert_eq;

    use crate::Namespace;

    use super::{Aliases, Query};

    fn run(aliases: &Aliases) -> Result<(), anyhow::Error> {
        let mut values = Vec::new();

        aliases.find(&Query::default(), |record| {
            values.push(record.unwrap().alias);
        })?;

        assert_eq!(
            values,
            vec![
                "a8e7e4cbd2fa521b45b23692b2dd601c",
                "NM_001304430.2",
                "U5AvKXlRSRwJgn/Zxsa286iO/sg",
                "53902f297951491c09827fd9c6c6b6f3a88efec8",
                "GS_5q5HZTCRudL17NTiv5Bn6th__0FrZH04",
            ]
        );

        Ok(())
    }

    #[test]
    fn smoke_test() -> Result<(), anyhow::Error> {
        let aliases = Aliases::new(&PathBuf::from("tests/data"), "aliases")?;
        run(&aliases)
    }

    #[test]
    fn try_clone() -> Result<(), anyhow::Error> {
        let aliases = Aliases::new(&PathBuf::from("tests/data"), "aliases")?;
        let second = aliases.try_clone()?;
        run(&second)
    }

    #[test]
    fn find_wildcard() -> Result<(), anyhow::Error> {
        let aliases = Aliases::new(&PathBuf::from("tests/data"), "aliases")?;

        let mut values = Vec::new();

        aliases.find(
            &Query {
                namespace: Some(Namespace("%".to_string())),
                alias: Some("%".to_string()),
                seqid: Some("%".to_string()),
                ..Default::default()
            },
            |record| {
                values.push(record.unwrap().alias);
            },
        )?;

        assert_eq!(
            values,
            vec![
                "a8e7e4cbd2fa521b45b23692b2dd601c",
                "NM_001304430.2",
                "U5AvKXlRSRwJgn/Zxsa286iO/sg",
                "53902f297951491c09827fd9c6c6b6f3a88efec8",
                "GS_5q5HZTCRudL17NTiv5Bn6th__0FrZH04",
            ]
        );

        Ok(())
    }

    #[test]
    fn find_no_wildcard() -> Result<(), anyhow::Error> {
        let aliases = Aliases::new(&PathBuf::from("tests/data"), "aliases")?;

        let mut values = Vec::new();

        aliases.find(
            &Query {
                namespace: None,
                alias: Some("NM_001304430.2".to_string()),
                seqid: None,
                ..Default::default()
            },
            |record| {
                values.push(record.unwrap().alias);
            },
        )?;

        assert_eq!(values, vec!["NM_001304430.2",]);

        Ok(())
    }
}
