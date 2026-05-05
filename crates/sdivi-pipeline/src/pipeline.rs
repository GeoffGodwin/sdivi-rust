//! [`Pipeline`] — five-stage analysis pipeline with full FS orchestration.
//!
//! This is the primary entry point for CLI users and Rust embedders that want
//! the full pipeline (parsing → graph → detection → patterns → snapshot).
//! For WASM consumers that supply their own extractors, use `sdivi-core::compute_*`
//! functions directly.

use std::path::Path;

use crate::cache::{load_cached_partition, save_cached_partition};
use crate::commit_extract::{commit_date_iso, extract_commit_tree, resolve_ref_to_sha};
use crate::error::PipelineError;
use crate::helpers::{
    build_edge_weight_map, compute_path_partition, graph_to_boundary_input, spec_to_boundary_input,
};
use crate::readers::{read_go_mod_prefix, read_tsconfig_paths};
use crate::store::{enforce_retention, write_snapshot};
pub use crate::time::current_timestamp;
use sdivi_config::{BoundarySpec, Config};
use sdivi_core::input::ChangeCouplingConfigInput;
use sdivi_detection::warm_start::CACHE_FILENAME;
use sdivi_detection::{run_leiden, run_leiden_with_weights, LeidenConfig};
use sdivi_graph::dependency_graph::build_dependency_graph_with_tsconfig;
use sdivi_graph::metrics::compute_metrics;
use sdivi_parsing::adapter::LanguageAdapter;
use sdivi_parsing::parse::parse_repository;
use sdivi_patterns::build_catalog;
use sdivi_snapshot::{assemble_snapshot, compute_delta, null_summary, DivergenceSummary, Snapshot};

/// Controls whether a snapshot is written to `.sdivi/snapshots/` after capture.
///
/// `Persist` is the default for `sdivi snapshot`; `EphemeralForCheck` is used
/// by `sdivi check --no-write` when the caller wants threshold evaluation without
/// polluting snapshot history.  This is the designated seam for any future
/// dry-run feature.
pub enum WriteMode {
    /// Write the snapshot to `.sdivi/snapshots/` and enforce retention.
    Persist,
    /// Compute the snapshot in memory only — no write, no retention.
    EphemeralForCheck,
}

/// The five-stage analysis pipeline.
///
/// `Pipeline` is the primary entry point for embedding sdivi in Rust programs.
/// Construction is cheap — the struct stores only the resolved [`Config`] and
/// the injected language adapters.  All analysis work happens inside
/// [`Pipeline::snapshot`].
///
/// For WASM / pure-compute consumers, use `sdivi_core::compute_*` functions
/// directly — they accept pre-extracted `*Input` structs and require no FS.
///
/// # Examples
///
/// ```rust
/// use sdivi_pipeline::Pipeline;
/// use sdivi_config::Config;
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
    /// use sdivi_pipeline::Pipeline;
    /// use sdivi_config::Config;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// ```
    pub fn new(config: Config, adapters: Vec<Box<dyn LanguageAdapter>>) -> Self {
        Pipeline { config, adapters }
    }

    /// Runs all five pipeline stages against `repo_root` and writes a snapshot.
    ///
    /// When `commit` is `Some(reference)`:
    /// - The reference is resolved to a full SHA via `git rev-parse`.
    /// - The tree at that SHA is extracted to a tempdir for parsing.
    /// - The snapshot's `timestamp` is the commit's commit-date (UTC), not the
    ///   wall-clock time of the invocation. The supplied `timestamp` argument is
    ///   **overridden** when `commit` is `Some`.
    /// - Change-coupling history is collected ending at the resolved SHA.
    /// - The tempdir is dropped before this function returns.
    ///
    /// A missing `.sdivi/boundaries.yaml` is **normal operation** (Rule 16).
    ///
    /// ## Errors
    ///
    /// Returns [`PipelineError`] on I/O failure, config errors, or git errors.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sdivi_pipeline::Pipeline;
    /// use sdivi_config::Config;
    /// use std::path::Path;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// let snapshot = pipeline.snapshot(Path::new("."), Some("abc1234"), "2026-04-29T00:00:00Z")?;
    /// println!("Communities: {}", snapshot.partition.community_count());
    /// # Ok::<(), sdivi_pipeline::PipelineError>(())
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
    /// [`WriteMode::Persist`] writes to `.sdivi/snapshots/` and enforces retention
    /// (identical to [`Pipeline::snapshot`]).  [`WriteMode::EphemeralForCheck`]
    /// computes the snapshot in memory only — used by `sdivi check --no-write`.
    ///
    /// See [`Pipeline::snapshot`] for the `--commit REF` semantics.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sdivi_pipeline::{Pipeline, WriteMode};
    /// use sdivi_config::Config;
    /// use std::path::Path;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// let snap = pipeline.snapshot_with_mode(
    ///     Path::new("."), None, "2026-04-29T00:00:00Z", WriteMode::EphemeralForCheck,
    /// )?;
    /// # Ok::<(), sdivi_pipeline::PipelineError>(())
    /// ```
    pub fn snapshot_with_mode(
        &self,
        repo_root: &Path,
        commit: Option<&str>,
        timestamp: &str,
        mode: WriteMode,
    ) -> Result<Snapshot, PipelineError> {
        // ── Resolve commit reference (if supplied) ───────────────────────
        // `_tempdir` is held here so the extracted tree lives for the full run.
        let (parse_root_buf, effective_sha, effective_ts, _tempdir) =
            if let Some(reference) = commit {
                let sha = resolve_ref_to_sha(repo_root, reference)?;
                let ts = commit_date_iso(repo_root, &sha)?;
                let td = extract_commit_tree(repo_root, &sha)?;
                let root = td.path().to_path_buf();
                (root, Some(sha), ts, Some(td))
            } else {
                (repo_root.to_path_buf(), None, timestamp.to_string(), None)
            };
        let parse_root = parse_root_buf.as_path();

        // ending_at drives the change-coupling window: None = HEAD, Some = REF.
        let ending_at = effective_sha.as_deref();
        // ── Stage 1: Parsing ─────────────────────────────────────────────
        let records = parse_repository(&self.config, parse_root, &self.adapters);
        tracing::info!(count = records.len(), "parsed {} files", records.len());

        // System Rule 7: exit 3 when all detected languages lack grammars.
        if records.is_empty() {
            let sdivi_dir = parse_root.join(".sdivi");
            let candidate_files = sdivi_parsing::walker::collect_files(&self.config, parse_root);
            let has_source_candidate = candidate_files
                .iter()
                .any(|p| p.extension().is_some() && !p.starts_with(&sdivi_dir));
            if has_source_candidate {
                return Err(PipelineError::NoGrammarsAvailable);
            }
        }
        // ── Stage 2: Graph ───────────────────────────────────────────────
        let go_mod_prefix = read_go_mod_prefix(parse_root);
        let tsconfig = read_tsconfig_paths(parse_root);
        let dg = build_dependency_graph_with_tsconfig(
            &records,
            go_mod_prefix.as_deref(),
            tsconfig.as_ref(),
        );
        let metrics = compute_metrics(&dg);
        // ── Change-coupling analysis ─────────────────────────────────────
        let cc_cfg = ChangeCouplingConfigInput {
            min_frequency: self.config.change_coupling.min_frequency,
            history_depth: self.config.change_coupling.history_depth,
        };
        // Always run against repo_root (has .git/); tempdir has no git history.
        let cc_events = crate::change_coupling::collect_cochange_events(
            repo_root,
            self.config.change_coupling.history_depth,
            ending_at,
        )
        .unwrap_or_else(|e| {
            tracing::warn!("change-coupling collection failed: {e}");
            vec![]
        });
        let change_coupling_result = if cc_events.is_empty() {
            None
        } else {
            sdivi_core::compute_change_coupling(&cc_events, &cc_cfg).ok()
        };

        // ── Stage 3: Detection ───────────────────────────────────────────
        let cache_path = repo_root.join(".sdivi").join(CACHE_FILENAME);
        let warm_partition = load_cached_partition(&cache_path);
        let leiden_cfg = LeidenConfig::from_sdivi_config(&self.config);
        let partition = if self.config.boundaries.weighted_edges {
            if let Some(ref ccr) = change_coupling_result {
                let weight_map = build_edge_weight_map(&dg, ccr);
                run_leiden_with_weights(&dg, &leiden_cfg, warm_partition.as_ref(), &weight_map)
            } else {
                run_leiden(&dg, &leiden_cfg, warm_partition.as_ref())
            }
        } else {
            run_leiden(&dg, &leiden_cfg, warm_partition.as_ref())
        };
        if let WriteMode::Persist = mode {
            save_cached_partition(&partition, &cache_path).map_err(PipelineError::SnapshotIo)?;
        }

        // ── Stage 4: Patterns ────────────────────────────────────────────
        let catalog = build_catalog(&records, &self.config.patterns);
        let pattern_metrics = sdivi_core::compute_pattern_metrics_from_catalog(&catalog);

        // ── Stage 5: Snapshot assembly ───────────────────────────────────
        let boundary_path = repo_root.join(&self.config.boundaries.spec_file);
        let boundary_spec: Option<BoundarySpec> =
            BoundarySpec::load(&boundary_path).unwrap_or(None);

        let violation_count = boundary_spec.as_ref().map_or(0, |spec| {
            let g_input = graph_to_boundary_input(&dg);
            let s_input = spec_to_boundary_input(spec);
            sdivi_core::compute_boundary_violations(&g_input, &s_input)
                .map(|r| r.violation_count)
                .unwrap_or_else(|e| {
                    tracing::warn!("boundary violation computation failed: {e}");
                    0
                })
        });

        let commit_label = effective_sha.as_deref();
        let boundary_count = boundary_spec.as_ref().map(|spec| spec.boundaries.len());
        let mut snapshot = assemble_snapshot(
            metrics,
            partition,
            catalog,
            pattern_metrics,
            boundary_count,
            &effective_ts,
            commit_label,
            change_coupling_result,
            violation_count,
        );
        snapshot.path_partition = compute_path_partition(&dg, &snapshot.partition);

        if let WriteMode::Persist = mode {
            let snapshot_dir = repo_root.join(&self.config.snapshots.dir);
            write_snapshot(&snapshot, &snapshot_dir).map_err(PipelineError::SnapshotIo)?;
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
    /// use sdivi_pipeline::Pipeline;
    /// use sdivi_snapshot::null_summary;
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

