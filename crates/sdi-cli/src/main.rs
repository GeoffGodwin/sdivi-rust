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
}

fn main() {
    let cli = Cli::parse();
    logging::init();

    let result = match cli.command {
        Some(Commands::Init) => commands::init::run(&cli.repo),
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
