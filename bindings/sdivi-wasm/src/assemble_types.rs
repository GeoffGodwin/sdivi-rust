//! Input type for the WASM `assemble_snapshot` export.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use crate::types::{WasmPatternInstanceInput, WasmPatternMetricsResult};

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
}
