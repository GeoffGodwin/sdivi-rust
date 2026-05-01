//! `sdi boundaries` — infer, ratify, and show module boundaries.

use anyhow::{Context, Result};
use clap::Subcommand;
use sdi_config::{BoundaryDef, BoundarySpec, Config};
use sdi_core::BoundaryProposal;
use sdi_pipeline::boundaries::infer_from_snapshots;
use sdi_pipeline::store::write_boundary_spec;

use std::path::Path;

/// Subcommands under `sdi boundaries`.
#[derive(Subcommand)]
pub enum BoundariesSubcmd {
    /// Propose module groupings from Leiden community detection history.
    Infer {
        /// Output format: `text` (default) or `json`.
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Write accepted groupings to `.sdi/boundaries.yaml`.
    Ratify,
    /// Show the current boundary specification.
    Show {
        /// Output format: `yaml` (default) or `json`.
        #[arg(long, default_value = "yaml")]
        format: String,
    },
}

/// Runs `sdi boundaries <subcmd>`.
pub fn run(subcmd: BoundariesSubcmd, repo_root: &Path, config: &Config) -> Result<()> {
    match subcmd {
        BoundariesSubcmd::Infer { format } => run_infer(repo_root, config, &format),
        BoundariesSubcmd::Ratify => run_ratify(repo_root, config),
        BoundariesSubcmd::Show { format } => run_show(repo_root, config, &format),
    }
}

fn run_infer(repo_root: &Path, config: &Config, format: &str) -> Result<()> {
    let snapshot_dir = repo_root.join(&config.snapshots.dir);
    let threshold = config.boundaries.stability_threshold;
    let result = infer_from_snapshots(&snapshot_dir, threshold)
        .with_context(|| "failed to read snapshot history for boundary inference")?;

    if result.proposals.is_empty() {
        eprintln!(
            "sdi boundaries infer: no stable communities found \
             (need at least {} snapshots with consistent partitions)",
            threshold
        );
        return Ok(());
    }

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&result)
                .context("failed to serialize inference result")?;
            println!("{json}");
        }
        _ => print_proposals_text(&result.proposals, result.partition_count),
    }
    Ok(())
}

fn run_ratify(repo_root: &Path, config: &Config) -> Result<()> {
    let snapshot_dir = repo_root.join(&config.snapshots.dir);
    let threshold = config.boundaries.stability_threshold;
    let result = infer_from_snapshots(&snapshot_dir, threshold)
        .with_context(|| "failed to read snapshot history for boundary inference")?;

    if result.proposals.is_empty() {
        eprintln!(
            "sdi boundaries ratify: no stable communities found — nothing to ratify"
        );
        return Ok(());
    }

    let boundaries: Vec<BoundaryDef> = result
        .proposals
        .iter()
        .map(|p| BoundaryDef {
            name: format!("community_{}", p.community_id),
            description: Some(format!(
                "Auto-inferred community (stable for {} snapshots)",
                p.stable_snapshots
            )),
            modules: p.node_ids.clone(),
            allow_imports_from: vec![],
        })
        .collect();

    let spec = BoundarySpec {
        version: None,
        boundaries,
    };

    let boundary_path = repo_root.join(&config.boundaries.spec_file);
    write_boundary_spec(&spec, &boundary_path)
        .with_context(|| format!("failed to write '{}'", boundary_path.display()))?;

    eprintln!(
        "sdi boundaries ratify: wrote {} boundaries to '{}'",
        spec.boundaries.len(),
        boundary_path.display()
    );
    Ok(())
}

fn run_show(repo_root: &Path, config: &Config, format: &str) -> Result<()> {
    let boundary_path = repo_root.join(&config.boundaries.spec_file);
    let spec = BoundarySpec::load(&boundary_path)
        .with_context(|| format!("failed to read '{}'", boundary_path.display()))?;

    match spec {
        None => {
            eprintln!(
                "sdi boundaries show: no boundary spec found at '{}' (missing is normal)",
                boundary_path.display()
            );
        }
        Some(spec) => match format {
            "json" => {
                let json = serde_json::to_string_pretty(&spec)
                    .context("failed to serialize boundary spec")?;
                println!("{json}");
            }
            _ => {
                print!("{}", spec.to_yaml());
            }
        },
    }
    Ok(())
}

fn print_proposals_text(proposals: &[BoundaryProposal], partition_count: usize) {
    println!("boundary proposals ({partition_count} partition(s) analysed):");
    for p in proposals {
        println!(
            "  community_{} ({} snapshots stable, {} files):",
            p.community_id, p.stable_snapshots, p.node_ids.len()
        );
        for node in &p.node_ids {
            println!("    {node}");
        }
    }
}
