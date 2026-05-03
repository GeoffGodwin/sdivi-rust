//! Input types for the WASM `assemble_snapshot` export.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::types::{WasmPatternInstanceInput, WasmPatternMetricsResult};

// ── change-coupling input (mirrors sdivi_core::ChangeCouplingResult) ─────────

/// A single file-pair entry for [`WasmChangeCouplingInput`].
///
/// Field names match `sdivi_core::CoChangePair` exactly — the serde round-trip
/// conversion in `assemble_snapshot` is field-name-based.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCoChangePairInput {
    /// The lexicographically smaller file path (`source < target`).
    pub source: String,
    /// The lexicographically larger file path.
    pub target: String,
    /// Co-change frequency: `cochange_count / commits_analyzed`.
    pub frequency: f64,
    /// Number of commits that touched both files.
    pub cochange_count: u32,
}

/// Change-coupling result passed into [`WasmAssembleSnapshotInput::change_coupling`].
///
/// Mirrors `sdivi_core::ChangeCouplingResult`. Typically the direct output of
/// [`crate::compute_change_coupling`] — pass it straight through without conversion.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmChangeCouplingInput {
    /// File pairs whose co-change frequency meets `min_frequency`.
    pub pairs: Vec<WasmCoChangePairInput>,
    /// Number of commits actually analyzed.
    pub commits_analyzed: u32,
    /// Count of unique file paths across all analyzed commits.
    pub distinct_files_touched: u32,
}

// ── assemble_snapshot input ───────────────────────────────────────────────────

/// Input to [`crate::assemble_snapshot`].
///
/// Collects the outputs of the three primary compute functions plus metadata.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmAssembleSnapshotInput {
    /// Ordered node IDs (determines numeric partition indices).
    pub node_ids: Vec<String>,
    /// node_id → community_id (from `detect_boundaries`).
    pub cluster_assignments: BTreeMap<String, u32>,
    /// community_id (string) → internal density (from `detect_boundaries`).
    pub internal_edge_density: BTreeMap<String, f64>,
    /// Modularity score from Leiden run.
    pub modularity: f64,
    /// Total node count.
    pub node_count: usize,
    /// Total edge count.
    pub edge_count: usize,
    /// Graph density.
    pub density: f64,
    /// Cycle count.
    pub cycle_count: usize,
    /// Top hub entries as `[node_id, out_degree]`.
    pub top_hubs: Vec<(String, usize)>,
    /// Number of weakly-connected components.
    pub component_count: usize,
    /// Pre-computed pattern metrics.
    pub pattern_metrics: WasmPatternMetricsResult,
    /// Raw pattern instances used to build the catalog (may be empty).
    pub pattern_instances: Vec<WasmPatternInstanceInput>,
    /// ISO-8601 UTC timestamp for the snapshot.
    pub timestamp: String,
    /// Optional git commit SHA.
    #[tsify(optional)]
    pub commit: Option<String>,
    /// Number of declared boundaries (sets intent_divergence when Some).
    #[tsify(optional)]
    pub boundary_count: Option<u32>,
    /// Seed used for the Leiden run that produced `cluster_assignments`.
    /// Defaults to 42 when absent (matches `LeidenConfigInput` default).
    #[tsify(optional)]
    pub leiden_seed: Option<u64>,
    /// Number of boundary violations (from `compute_boundary_violations`).
    /// When `Some`, sets `intent_divergence.violation_count` in the snapshot.
    #[tsify(optional)]
    pub violation_count: Option<u32>,

    /// Change-coupling result (from [`crate::compute_change_coupling`]).
    /// When `Some`, populates the `change_coupling` field of the assembled snapshot,
    /// identical to what `sdivi-pipeline` produces for native callers.
    #[serde(default)]
    #[tsify(optional)]
    pub change_coupling: Option<WasmChangeCouplingInput>,
}
