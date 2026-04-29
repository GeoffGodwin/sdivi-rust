mod commands;
mod logging;
mod output;

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use sdi_core::ExitCode;

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
/// All other errors → [`ExitCode::RuntimeError`] (1).
fn error_exit_code(e: &anyhow::Error) -> ExitCode {
    if e.downcast_ref::<sdi_config::ConfigError>().is_some() {
        ExitCode::ConfigError
    } else {
        ExitCode::RuntimeError
    }
}
