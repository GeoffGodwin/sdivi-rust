//! `sdi boundaries` — M09 stubs; full implementation in Milestone 10.

use anyhow::Result;
use clap::Subcommand;

/// Subcommands under `sdi boundaries`.
///
/// All subcommands are stubs in M09 and will be implemented in Milestone 10.
#[derive(Subcommand)]
pub enum BoundariesSubcmd {
    /// Infer boundary proposals from Leiden community detection.
    Infer,
    /// Ratify inferred boundaries by writing them to `.sdi/boundaries.yaml`.
    Ratify,
    /// Show the current boundary specification and any violations.
    Show,
}

/// Runs `sdi boundaries <subcmd>` — M09 stub.
///
/// Each subcommand prints a "not implemented until M10" message to stderr
/// and exits 0, keeping CI scripts that probe available commands working.
pub fn run(subcmd: BoundariesSubcmd) -> Result<()> {
    let name = match subcmd {
        BoundariesSubcmd::Infer => "infer",
        BoundariesSubcmd::Ratify => "ratify",
        BoundariesSubcmd::Show => "show",
    };
    eprintln!("sdi boundaries {name}: not implemented until M10");
    Ok(())
}
