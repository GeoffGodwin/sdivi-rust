//! `sdi check` — capture a snapshot, compare to prior, and evaluate thresholds.

use std::path::Path;

use anyhow::{Context, Result};
use sdi_config::Config;
use sdi_core::input::{ThresholdOverrideInput, ThresholdsInput};
use sdi_core::{ExitCode, compute_thresholds_check};
use sdi_pipeline::store::latest_snapshot;
use sdi_pipeline::{Pipeline, WriteMode, current_timestamp};

use crate::output;

/// Runs `sdi check`.
///
/// Captures a fresh snapshot, computes the delta against the most recent stored
/// prior, evaluates thresholds via [`sdi_core::compute_thresholds_check`], and
/// prints results.  Returns [`ExitCode::ThresholdExceeded`] (10) if any
/// threshold was exceeded, [`ExitCode::Success`] (0) otherwise.
///
/// When `no_write` is true, the fresh snapshot is not persisted and retention
/// is not enforced — useful for CI gates that must not pollute history.
///
/// # Errors
///
/// Returns an error if the pipeline fails or snapshot I/O fails.
pub fn run(
    repo_root: &Path,
    config: &Config,
    no_write: bool,
    format: &str,
) -> Result<ExitCode> {
    let snapshot_dir = repo_root.join(&config.snapshots.dir);
    let prior = latest_snapshot(&snapshot_dir)
        .with_context(|| format!("failed to read snapshot dir: {}", snapshot_dir.display()))?;

    let pipeline = Pipeline::new(config.clone(), super::all_adapters());
    let timestamp = current_timestamp();
    let mode = if no_write {
        WriteMode::EphemeralForCheck
    } else {
        WriteMode::Persist
    };

    eprintln!("sdi: checking repository at {}", repo_root.display());
    let current = pipeline.snapshot_with_mode(repo_root, None, &timestamp, mode)?;

    let summary = Pipeline::delta(prior.as_ref(), &current);

    let today = chrono::Local::now().date_naive();
    let thresholds = thresholds_input(config, today);
    let check_result = compute_thresholds_check(&summary, &thresholds);

    match format {
        "json" => output::json::print_check(&check_result, &summary)?,
        _ => output::text::print_check(&check_result, &summary),
    }

    if check_result.breached {
        Ok(ExitCode::ThresholdExceeded)
    } else {
        Ok(ExitCode::Success)
    }
}

/// Builds [`ThresholdsInput`] from [`Config`] with the caller-supplied `today`.
///
/// The clock is read in the CLI, not in `sdi-core` (Rule 21).
fn thresholds_input(config: &Config, today: chrono::NaiveDate) -> ThresholdsInput {
    let t = &config.thresholds;
    let overrides = t
        .overrides
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                ThresholdOverrideInput {
                    pattern_entropy_rate: v.pattern_entropy_rate,
                    convention_drift_rate: v.convention_drift_rate,
                    coupling_delta_rate: v.coupling_delta_rate,
                    boundary_violation_rate: v.boundary_violation_rate,
                    expires: v.expires.clone(),
                },
            )
        })
        .collect();

    ThresholdsInput {
        pattern_entropy_rate: t.pattern_entropy_rate,
        convention_drift_rate: t.convention_drift_rate,
        coupling_delta_rate: t.coupling_delta_rate,
        boundary_violation_rate: t.boundary_violation_rate,
        overrides,
        today,
    }
}
