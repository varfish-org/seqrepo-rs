//! Command line interface to the `seqrepo` crate.

use clap::{arg, command, Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use textwrap::wrap;
use tracing::debug;

use seqrepo::{
    AliasDbRecord, Namespace as LibNamespace, NamespacedAlias as LibNamespacedAlias, Query, SeqRepo,
};

/// Commonly used command line arguments.
#[derive(Parser, Debug)]
pub struct CommonArgs {
    /// Verbosity of the program
    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    /// Root directory
    #[arg(
        short,
        long,
        env = "SEQREPO_ROOT_DIR",
        default_value = "~/hgvs-rs-data/seqrepo-data"
    )]
    pub root_directory: String,
}

/// CLI parser based on clap.
#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "SeqRepo access written in Rust",
    long_about = "(Read-only) access to SeqRepo data from Rust"
)]
struct Cli {
    /// Commonly used arguments
    #[command(flatten)]
    common: CommonArgs,

    /// The sub command to run
    #[command(subcommand)]
    command: Commands,
}

/// Enum supporting the parsing of top-level commands.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
enum Commands {
    /// "export" sub command
    Export(ExportArgs),
}

/// Enum for selecting the namespace on the command line.
#[derive(ValueEnum, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Namespace {
    Refseq,
    Ensembl,
    Lrg,
    Sha512t24u,
    Ga4gh,
}

impl From<Namespace> for LibNamespace {
    fn from(value: Namespace) -> Self {
        match value {
            Namespace::Refseq => LibNamespace("NCBI".to_string()),
            Namespace::Ensembl => LibNamespace("Ensembl".to_string()),
            Namespace::Lrg => LibNamespace("Lrg".to_string()),
            Namespace::Sha512t24u => LibNamespace("".to_string()),
            Namespace::Ga4gh => LibNamespace("".to_string()),
        }
    }
}

/// A pair of namespace and alias as read from the command line.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct NamespacedAlias {
    pub namespace: Namespace,
    pub alias: String,
}

impl From<NamespacedAlias> for LibNamespacedAlias {
    fn from(value: NamespacedAlias) -> Self {
        LibNamespacedAlias {
            alias: match value.namespace {
                Namespace::Refseq | Namespace::Ensembl | Namespace::Lrg => value.alias,
                Namespace::Sha512t24u => format!("GS_{}", value.alias),
                Namespace::Ga4gh => format!("GS_{}", &value.alias[3..]),
            },
            namespace: value.namespace.into(),
        }
    }
}

/// Parsing of "export" subcommand
#[derive(Debug, Args)]
struct ExportArgs {
    /// The namespace to use.
    #[arg(short, long, value_enum, rename_all = "lower")]
    pub namespace: Option<Namespace>,
    /// The instance name to use.
    #[arg(short, long, default_value = "latest")]
    pub instance_name: String,
    /// The sequence aliases to query for.
    #[arg()]
    pub aliases: Vec<String>,
}

/// Implementation of "export" command.
fn main_export(common_args: &CommonArgs, args: &ExportArgs) -> Result<(), anyhow::Error> {
    debug!("common_args = {:?}", &common_args);
    debug!("args = {:?}", &args);

    let seq_repo = SeqRepo::new(&common_args.root_directory, &args.instance_name)?;
    let alias_db = seq_repo.alias_db();

    let mut query = Query {
        namespace: args.namespace.as_ref().map(|namespace| (*namespace).into()),
        ..Default::default()
    };

    let mut group: Vec<AliasDbRecord> = Vec::new();

    fn print_and_clear_group(seq_repo: &SeqRepo, group: &mut Vec<AliasDbRecord>) {
        if !group.is_empty() {
            let seq = seq_repo
                .fetch_sequence(&seqrepo::AliasOrSeqId::SeqId(group[0].seqid.clone()))
                .unwrap();
            group.sort_by(|a, b| {
                let (LibNamespace(a), LibNamespace(b)) = (&a.namespace, &b.namespace);
                a.partial_cmp(b).unwrap()
            });
            let metas = group
                .iter()
                .map(|record| match &record.namespace {
                    LibNamespace(namespace) => format!("{}:{}", namespace, record.alias),
                })
                .collect::<Vec<_>>();

            println!(">{}", metas.join(" "));
            for line in wrap(&seq, 100) {
                println!("{line}");
            }

            group.clear();
        }
    }

    let mut handle_record = |record: Result<AliasDbRecord, anyhow::Error>| {
        let record = record.unwrap();
        if !group.is_empty() && group[0].seqid != record.seqid {
            print_and_clear_group(&seq_repo, &mut group);
        }
        group.push(record);
    };

    if args.aliases.is_empty() {
        alias_db.find(&query, &mut handle_record)?;
    } else {
        for alias in &args.aliases {
            query.alias = Some(alias.clone());
            alias_db.find(&query, &mut handle_record)?;
        }
    }

    print_and_clear_group(&seq_repo, &mut group);

    Ok(())
}

pub fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Build a tracing subscriber according to the configuration in `cli.common`.
    let collector = tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(match cli.common.verbose.log_level() {
            Some(level) => match level {
                log::Level::Error => tracing::Level::ERROR,
                log::Level::Warn => tracing::Level::WARN,
                log::Level::Info => tracing::Level::INFO,
                log::Level::Debug => tracing::Level::DEBUG,
                log::Level::Trace => tracing::Level::TRACE,
            },
            None => tracing::Level::INFO,
        })
        .compact()
        .finish();

    tracing::subscriber::with_default(collector, || {
        match &cli.command {
            Commands::Export(args) => {
                main_export(&cli.common, args)?;
            }
        }

        Ok::<(), anyhow::Error>(())
    })?;

    debug!("All done! Have a nice day.");

    Ok(())
}

#[cfg(test)]
mod test {
    use clap_verbosity_flag::Verbosity;

    use super::main_export;
    use crate::{CommonArgs, ExportArgs};

    #[test]
    fn run_cmd() -> Result<(), anyhow::Error> {
        main_export(
            &CommonArgs {
                verbose: Verbosity::new(0, 0),
                root_directory: "tests/data/seqrepo".to_string(),
            },
            &ExportArgs {
                namespace: None,
                instance_name: "latest".to_string(),
                aliases: vec!["XR_001757199.1".to_string()],
            },
        )
    }
}
