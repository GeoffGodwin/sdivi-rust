mod commands;
mod logging;
mod output;

use clap::Parser;

/// Structural Divergence Indexer — measure structural drift in your codebase.
#[derive(Parser)]
#[command(name = "sdi", version, about, long_about = None)]
struct Cli {}

fn main() {
    let _cli = Cli::parse();
    logging::init();
}
