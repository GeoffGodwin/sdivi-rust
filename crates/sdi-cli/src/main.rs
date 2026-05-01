mod commands;
mod logging;
mod output;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sdi_core::ExitCode;

use commands::boundaries::BoundariesSubcmd;

/// Structural Divergence Indexer — measure structural drift in your codebase.
#[derive(Parser)]
#[command(name = "sdi", version, about, long_about = None)]
struct Cli {
    /// Repository root (default: current directory).
    #[arg(long, default_value = ".")]
    repo: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize `.sdi/` and write a default config.
    Init,
    /// Build and display the pattern catalog for the repository.
    Catalog {
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Capture a snapshot of the repository's current structural state.
    Snapshot {
        /// Git commit SHA to record (optional).
        #[arg(long)]
        commit: Option<String>,
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Compare two snapshots and display the divergence summary.
    Diff {
        /// Path to the previous (older) snapshot JSON file.
        prev: PathBuf,
        /// Path to the current (newer) snapshot JSON file.
        curr: PathBuf,
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Capture a snapshot, compare to prior, and exit 10 if thresholds are exceeded.
    Check {
        /// Skip writing the new snapshot to `.sdi/snapshots/` (retention not enforced).
        #[arg(long)]
        no_write: bool,
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Show trend statistics across stored snapshots.
    Trend {
        /// Number of most-recent snapshots to include (default: all).
        #[arg(long)]
        last: Option<usize>,
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Inspect a stored snapshot.
    Show {
        /// Snapshot id (filename stem without `.json`); defaults to the latest.
        id: Option<String>,
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Manage declared module boundaries (infer, ratify, show).
    Boundaries {
        #[command(subcommand)]
        subcmd: BoundariesSubcmd,
    },
}

fn main() {
    let cli = Cli::parse();
    logging::init();

    let config = match sdi_config::load_or_default(&cli.repo) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("sdi: error: {e:#}");
            std::process::exit(ExitCode::ConfigError.as_i32());
        }
    };

    // `check` returns ExitCode directly (may be 10); handle it before the
    // standard Result<()> dispatch so exit-10 is not conflated with an error.
    if let Some(Commands::Check { no_write, format }) = &cli.command {
        match commands::check::run(&cli.repo, &config, *no_write, format) {
            Ok(code) => std::process::exit(code.as_i32()),
            Err(e) => {
                eprintln!("sdi: error: {e:#}");
                std::process::exit(error_exit_code(&e).as_i32());
            }
        }
    }

    let result = match cli.command {
        Some(Commands::Init) => commands::init::run(&cli.repo),
        Some(Commands::Catalog { format }) => {
            commands::catalog::run(&cli.repo, &config, &format)
        }
        Some(Commands::Snapshot { commit, format }) => {
            commands::snapshot::run(&cli.repo, &config, commit.as_deref(), &format)
        }
        Some(Commands::Diff { prev, curr, format }) => {
            commands::diff::run(&prev, &curr, &format)
        }
        Some(Commands::Check { .. }) => unreachable!("handled above"),
        Some(Commands::Trend { last, format }) => {
            commands::trend::run(&cli.repo, &config, last, &format)
        }
        Some(Commands::Show { id, format }) => {
            commands::show::run(&cli.repo, &config, id.as_deref(), &format)
        }
        Some(Commands::Boundaries { subcmd }) => {
            commands::boundaries::run(subcmd, &cli.repo, &config)
        }
        None => {
            eprintln!("sdi: no subcommand given — try `sdi --help`");
            return;
        }
    };

    if let Err(e) = result {
        let code = error_exit_code(&e);
        eprintln!("sdi: error: {e:#}");
        std::process::exit(code.as_i32());
    }
}

/// Maps an `anyhow::Error` to the appropriate [`ExitCode`].
///
/// `ConfigError` sources → [`ExitCode::ConfigError`] (2).
/// `PipelineError::NoGrammarsAvailable` → [`ExitCode::AnalysisError`] (3).
/// All other errors → [`ExitCode::RuntimeError`] (1).
fn error_exit_code(e: &anyhow::Error) -> ExitCode {
    if e.downcast_ref::<sdi_config::ConfigError>().is_some() {
        return ExitCode::ConfigError;
    }
    if let Some(pe) = e.downcast_ref::<sdi_pipeline::PipelineError>() {
        if matches!(pe, sdi_pipeline::PipelineError::NoGrammarsAvailable) {
            return ExitCode::AnalysisError;
        }
    }
    ExitCode::RuntimeError
}
