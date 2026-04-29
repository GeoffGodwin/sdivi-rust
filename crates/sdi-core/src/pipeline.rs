//! [`Pipeline`] — composition of all five analysis stages.
//!
//! The pipeline wires together parsing, graph construction, community
//! detection, pattern cataloging, and snapshot assembly into a single
//! cohesive entry point.  Language adapters are injected by the caller
//! (typically `sdi-cli`) so that `sdi-core` carries no compile-time
//! dependency on language-specific grammar crates.

use std::path::Path;

use sdi_config::{BoundarySpec, Config};
use sdi_detection::warm_start::{load_cached_partition, save_cached_partition, CACHE_FILENAME};
use sdi_detection::{LeidenConfig, run_leiden};
use sdi_graph::dependency_graph::build_dependency_graph;
use sdi_graph::metrics::compute_metrics;
use sdi_parsing::adapter::LanguageAdapter;
use sdi_parsing::parse::parse_repository;
use sdi_patterns::build_catalog;
use sdi_snapshot::{
    DivergenceSummary, Snapshot, build_snapshot, compute_delta, enforce_retention, null_summary,
    write_snapshot,
};

use crate::error::AnalysisError;

/// The five-stage analysis pipeline.
///
/// `Pipeline` is the primary entry point for embedding sdi-core in Rust
/// programs and language bindings.  Construction is cheap — the struct
/// stores only the resolved [`Config`] and the injected language adapters.
/// All analysis work happens inside [`Pipeline::snapshot`].
///
/// # Examples
///
/// ```rust
/// use sdi_core::Pipeline;
/// use sdi_config::Config;
///
/// let pipeline = Pipeline::new(Config::default(), vec![]);
/// // `pipeline` is now ready to call `snapshot` against a repository.
/// ```
pub struct Pipeline {
    config: Config,
    adapters: Vec<Box<dyn LanguageAdapter>>,
}

impl Pipeline {
    /// Creates a new `Pipeline` from a resolved configuration and a set of
    /// language adapters.
    ///
    /// Construction is **O(1)** — no file I/O, no parsing, no allocation
    /// beyond storing the arguments.  The caller is responsible for
    /// constructing and passing the adapters that match the languages present
    /// in the target repository.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_core::Pipeline;
    /// use sdi_config::Config;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// ```
    pub fn new(config: Config, adapters: Vec<Box<dyn LanguageAdapter>>) -> Self {
        Pipeline { config, adapters }
    }

    /// Runs all five pipeline stages against `repo_root` and writes a snapshot.
    ///
    /// ## Stage order
    ///
    /// 1. **Parsing** — walks the repository using the configured
    ///    [`Config`] excludes and calls each registered [`LanguageAdapter`].
    /// 2. **Graph** — builds a `petgraph`-backed [`DependencyGraph`] and
    ///    computes [`GraphMetrics`].
    /// 3. **Detection** — loads a warm-start partition from
    ///    `.sdi/cache/partition.json` (if present), runs the native Leiden
    ///    algorithm, and persists the result back to cache.
    /// 4. **Patterns** — builds a [`PatternCatalog`] from parsing output.
    /// 5. **Snapshot** — assembles all stage outputs into a [`Snapshot`],
    ///    writes it atomically to the configured snapshot directory, and
    ///    enforces the retention limit.
    ///
    /// A missing `.sdi/boundaries.yaml` is **normal operation** (Rule 16):
    /// `intent_divergence` will be absent from the returned snapshot.
    ///
    /// ## Errors
    ///
    /// Returns [`AnalysisError`] on I/O failure during snapshot write or
    /// partition cache save, or on config-level errors surfaced during
    /// boundary spec loading.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sdi_core::Pipeline;
    /// use sdi_config::Config;
    /// use std::path::Path;
    ///
    /// let pipeline = Pipeline::new(Config::default(), vec![]);
    /// let snapshot = pipeline.snapshot(
    ///     Path::new("."),
    ///     Some("abc1234"),
    ///     "2026-04-29T00:00:00Z",
    /// )?;
    /// println!("Communities: {}", snapshot.partition.community_count());
    /// # Ok::<(), sdi_core::AnalysisError>(())
    /// ```
    pub fn snapshot(
        &self,
        repo_root: &Path,
        commit: Option<&str>,
        timestamp: &str,
    ) -> Result<Snapshot, AnalysisError> {
        // ── Stage 1: Parsing ─────────────────────────────────────────────
        let records = parse_repository(&self.config, repo_root, &self.adapters);
        tracing::info!(count = records.len(), "parsed {} files", records.len());

        // ── Stage 2: Graph ───────────────────────────────────────────────
        let dg = build_dependency_graph(&records);
        let metrics = compute_metrics(&dg);

        // ── Stage 3: Detection ───────────────────────────────────────────
        let cache_path = repo_root.join(".sdi").join(CACHE_FILENAME);
        let warm_start = load_cached_partition(&cache_path);
        let leiden_cfg = LeidenConfig::from_sdi_config(&self.config);
        let partition = run_leiden(&dg, &leiden_cfg, warm_start.as_ref());
        save_cached_partition(&partition, &cache_path)
            .map_err(AnalysisError::SnapshotIo)?;

        // ── Stage 4: Patterns ────────────────────────────────────────────
        let catalog = build_catalog(&records, &self.config.patterns);

        // ── Stage 5: Snapshot assembly ───────────────────────────────────
        let boundary_path = repo_root.join(&self.config.boundaries.spec_file);
        // Missing boundary spec is normal operation (Rule 16).
        let boundary_spec: Option<BoundarySpec> = BoundarySpec::load(&boundary_path)
            .unwrap_or(None);

        let snapshot = build_snapshot(
            metrics,
            partition,
            catalog,
            boundary_spec.as_ref(),
            timestamp,
            commit,
        );

        let snapshot_dir = repo_root.join(&self.config.snapshots.dir);
        write_snapshot(&snapshot, &snapshot_dir)
            .map_err(AnalysisError::SnapshotIo)?;
        enforce_retention(&snapshot_dir, self.config.snapshots.retention)
            .map_err(AnalysisError::SnapshotIo)?;

        Ok(snapshot)
    }

    /// Computes the per-dimension divergence between two snapshots.
    ///
    /// This is a **pure, static** function — it performs no I/O, reads no
    /// globals, and uses no clock.  The same two snapshots always produce the
    /// same [`DivergenceSummary`].
    ///
    /// When `prev` is `None` (first snapshot — no prior to compare against),
    /// every field of the returned summary is `null`.  When `prev` is `Some`,
    /// all four dimensions are computed.  `Some(0)` / `Some(0.0)` means "no
    /// change observed", which is intentionally distinct from `None` (Rule 14).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use sdi_core::{Pipeline, DivergenceSummary, null_summary};
    ///
    /// // First snapshot: no prior available — all fields are null.
    /// let first_run = null_summary();
    /// assert!(first_run.coupling_delta.is_none());
    /// assert!(first_run.community_count_delta.is_none());
    /// ```
    pub fn delta(prev: Option<&Snapshot>, curr: &Snapshot) -> DivergenceSummary {
        match prev {
            None => null_summary(),
            Some(p) => compute_delta(p, curr),
        }
    }
}
