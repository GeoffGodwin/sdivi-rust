//! [`Pipeline`] — five-stage analysis pipeline with full FS orchestration.
//!
//! This is the primary entry point for CLI users and Rust embedders that want
//! the full pipeline (parsing → graph → detection → patterns → snapshot).
//! For WASM consumers that supply their own extractors, use `sdi-core::compute_*`
//! functions directly.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use sdi_config::{BoundarySpec, Config};
use sdi_detection::warm_start::CACHE_FILENAME;
use sdi_detection::{LeidenConfig, run_leiden};
use sdi_graph::dependency_graph::build_dependency_graph;
use sdi_graph::metrics::compute_metrics;
use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::parse::parse_repository;
use sdi_patterns::build_catalog;
use sdi_snapshot::{
    DivergenceSummary, Snapshot, assemble_snapshot, compute_delta, null_summary,
};
use sdi_snapshot::snapshot::PatternMetricsResult;

use crate::cache::{load_cached_partition, save_cached_partition};
use crate::error::PipelineError;
use crate::store::{write_snapshot, enforce_retention};

/// Controls whether a snapshot is written to `.sdi/snapshots/` after capture.
///
/// `Persist` is the default for `sdi snapshot`; `EphemeralForCheck` is used
/// by `sdi check --no-write` when the caller wants threshold evaluation without
/// polluting snapshot history.  This is the designated seam for any future
/// dry-run feature.
pub enum WriteMode {
    /// Write the snapshot to `.sdi/snapshots/` and enforce retention.
    Persist,
    /// Compute the snapshot in memory only — no write, no retention.
    EphemeralForCheck,
}

/// The five-stage analysis pipeline.
///
/// `Pipeline` is the primary entry point for embedding sdi in Rust programs.
/// Construction is cheap — the struct stores only the resolved [`Config`] and
/// the injected language adapters.  All analysis work happens inside
/// [`Pipeline::snapshot`].
///
/// For WASM / pure-compute consumers, use `sdi_core::compute_*` functions
/// directly — they accept pre-extracted `*Input` structs and require no FS.
///
/// # Examples
///
/// ```rust
/// use sdi_pipeline::Pipeline;
/// use sdi_config::Config;
///
/// let pipeline = Pipeline::new(Config::default(), vec![]);
/// ```
pub struct Pipeline {
    config: Config,
    adapters: Vec<Box<dyn LanguageAdapter>>,
}

impl Pipeline {
    /// Creates a new `Pipeline` from a resolved configuration and language adapters.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_pipeline::Pipeline;
    /// use sdi_config::Config;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// ```
    pub fn new(config: Config, adapters: Vec<Box<dyn LanguageAdapter>>) -> Self {
        Pipeline { config, adapters }
    }

    /// Runs all five pipeline stages against `repo_root` and writes a snapshot.
    ///
    /// A missing `.sdi/boundaries.yaml` is **normal operation** (Rule 16).
    ///
    /// ## Errors
    ///
    /// Returns [`PipelineError`] on I/O failure or config errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sdi_pipeline::Pipeline;
    /// use sdi_config::Config;
    /// use std::path::Path;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// let snapshot = pipeline.snapshot(Path::new("."), Some("abc1234"), "2026-04-29T00:00:00Z")?;
    /// println!("Communities: {}", snapshot.partition.community_count());
    /// # Ok::<(), sdi_pipeline::PipelineError>(())
    /// ```
    pub fn snapshot(
        &self,
        repo_root: &Path,
        commit: Option<&str>,
        timestamp: &str,
    ) -> Result<Snapshot, PipelineError> {
        self.snapshot_with_mode(repo_root, commit, timestamp, WriteMode::Persist)
    }

    /// Runs all five pipeline stages; write behaviour is controlled by `mode`.
    ///
    /// [`WriteMode::Persist`] writes to `.sdi/snapshots/` and enforces retention
    /// (identical to [`Pipeline::snapshot`]).  [`WriteMode::EphemeralForCheck`]
    /// computes the snapshot in memory only — used by `sdi check --no-write`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sdi_pipeline::{Pipeline, WriteMode};
    /// use sdi_config::Config;
    /// use std::path::Path;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// let snap = pipeline.snapshot_with_mode(
    ///     Path::new("."), None, "2026-04-29T00:00:00Z", WriteMode::EphemeralForCheck,
    /// )?;
    /// # Ok::<(), sdi_pipeline::PipelineError>(())
    /// ```
    pub fn snapshot_with_mode(
        &self,
        repo_root: &Path,
        commit: Option<&str>,
        timestamp: &str,
        mode: WriteMode,
    ) -> Result<Snapshot, PipelineError> {
        // ── Stage 1: Parsing ─────────────────────────────────────────────
        let records = parse_repository(&self.config, repo_root, &self.adapters);
        tracing::info!(count = records.len(), "parsed {} files", records.len());

        // ── Stage 2: Graph ───────────────────────────────────────────────
        let dg = build_dependency_graph(&records);
        let metrics = compute_metrics(&dg);

        // ── Stage 3: Detection ───────────────────────────────────────────
        let cache_path = repo_root.join(".sdi").join(CACHE_FILENAME);
        let warm_partition = load_cached_partition(&cache_path);
        let leiden_cfg = LeidenConfig::from_sdi_config(&self.config);
        let partition = run_leiden(&dg, &leiden_cfg, warm_partition.as_ref());
        save_cached_partition(&partition, &cache_path)
            .map_err(PipelineError::SnapshotIo)?;

        // ── Stage 4: Patterns ────────────────────────────────────────────
        let catalog = build_catalog(&records, &self.config.patterns);
        let pattern_metrics = compute_pattern_metrics_from_catalog(&catalog);

        // ── Stage 5: Snapshot assembly ───────────────────────────────────
        let boundary_path = repo_root.join(&self.config.boundaries.spec_file);
        let boundary_spec: Option<BoundarySpec> = BoundarySpec::load(&boundary_path)
            .unwrap_or(None);

        let snapshot = assemble_snapshot(
            metrics,
            partition,
            catalog,
            pattern_metrics,
            boundary_spec.as_ref(),
            timestamp,
            commit,
        );

        if let WriteMode::Persist = mode {
            let snapshot_dir = repo_root.join(&self.config.snapshots.dir);
            write_snapshot(&snapshot, &snapshot_dir)
                .map_err(PipelineError::SnapshotIo)?;
            enforce_retention(&snapshot_dir, self.config.snapshots.retention)
                .map_err(PipelineError::SnapshotIo)?;
        }

        Ok(snapshot)
    }

    /// Computes the per-dimension divergence between two snapshots.
    ///
    /// Pure function — no I/O, no clock.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_pipeline::Pipeline;
    /// use sdi_snapshot::null_summary;
    ///
    /// let first_run = null_summary();
    /// assert!(first_run.coupling_delta.is_none());
    /// ```
    pub fn delta(prev: Option<&Snapshot>, curr: &Snapshot) -> DivergenceSummary {
        match prev {
            None => null_summary(),
            Some(p) => compute_delta(p, curr),
        }
    }
}

/// Builds [`PatternMetricsResult`] from a catalog using the same entropy logic
/// as `sdi_core::compute_pattern_metrics`.
fn compute_pattern_metrics_from_catalog(
    catalog: &sdi_patterns::PatternCatalog,
) -> PatternMetricsResult {
    use sdi_patterns::compute_entropy;

    let entropy_per_category: std::collections::BTreeMap<String, f64> = catalog
        .entries
        .iter()
        .map(|(cat, stats)| (cat.clone(), compute_entropy(stats)))
        .collect();

    let total_entropy: f64 = entropy_per_category.values().sum();

    let convention_drift = if catalog.entries.is_empty() {
        0.0
    } else {
        let sum: f64 = catalog.entries.values().map(|cat_stats| {
            let distinct = cat_stats.len() as f64;
            let total: f64 = cat_stats.values().map(|s| s.count as f64).sum();
            if total > 0.0 { distinct / total } else { 0.0 }
        }).sum();
        sum / catalog.entries.len() as f64
    };

    PatternMetricsResult {
        entropy_per_category,
        total_entropy,
        convention_drift,
    }
}

/// Returns the current UTC time as an ISO 8601 string.
pub fn current_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    unix_to_iso8601(secs)
}

fn unix_to_iso8601(secs: u64) -> String {
    let secs = secs as i64;
    let days_since_epoch = secs / 86400;
    let time_secs = secs % 86400;
    let j = days_since_epoch + 2440588;
    let f = j + 1401 + (((4 * j + 274277) / 146097) * 3) / 4 - 38;
    let e = 4 * f + 3;
    let g = (e % 1461) / 4;
    let h = 5 * g + 2;
    let day = (h % 153) / 5 + 1;
    let month = (h / 153 + 2) % 12 + 1;
    let year = e / 1461 - 4716 + (14 - month) / 12;
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day,
        time_secs / 3600, (time_secs % 3600) / 60, time_secs % 60,
    )
}
