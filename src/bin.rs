use clap::{arg, command, Args, Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::info;

use seqrepo::{
    AliasDbRecord, Namespace as LibNamespace, NamespacedAlias as LibNamespacedAlias, Query,
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

fn print_record(record: Result<AliasDbRecord, anyhow::Error>) {
    info!("{:?}", record.expect("problem loading record"));
}

/// Implementation of "export" command.
fn main_export(common_args: &CommonArgs, args: &ExportArgs) -> Result<(), anyhow::Error> {
    info!("common_args = {:?}", &common_args);
    info!("args = {:?}", &args);

    let seq_repo = seqrepo::SeqRepo::new(&common_args.root_directory, &args.instance_name)?;
    let alias_db = seq_repo.alias_db();

    let mut query = Query {
        namespace: args.namespace.as_ref().map(|namespace| (*namespace).into()),
        ..Default::default()
    };

    if args.aliases.is_empty() {
        alias_db.find(&query, print_record)?;
    } else {
        for alias in &args.aliases {
            query.alias = Some(alias.clone());
            alias_db.find(&query, print_record)?;
        }
    }

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

    info!("All done! Have a nice day.");

    Ok(())
}
