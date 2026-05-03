//! Tsify-derived wrapper types for all sdivi-wasm exported function signatures.
//!
//! Each type mirrors the serde format of its sdivi-core counterpart so that a
//! serde_json round-trip converts between them without explicit From impls.
//! BTreeMap keys that are numeric in sdivi-core (usize/u32) are represented as
//! String here; serde_json correctly round-trips both representations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

/// A single node in a [`WasmDependencyGraphInput`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmNodeInput {
    pub id: String,
    pub path: String,
    pub language: String,
}

/// A directed edge between two nodes.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmEdgeInput {
    pub source: String,
    pub target: String,
}

/// Input dependency graph for pure-compute functions.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmDependencyGraphInput {
    pub nodes: Vec<WasmNodeInput>,
    pub edges: Vec<WasmEdgeInput>,
}

/// Quality function for Leiden community detection.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum WasmQualityFunction {
    Modularity,
    Cpm,
}

/// Leiden algorithm configuration.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmLeidenConfigInput {
    pub seed: u64,
    pub gamma: f64,
    pub iterations: usize,
    pub quality: WasmQualityFunction,
    /// Per-edge weights keyed `"source:target"` (first colon splits source/target). `None` = unweighted.
    /// Weights must be `>= 0.0` and finite. Edges absent from the graph are silently ignored.
    #[serde(default)]
    #[tsify(optional)]
    pub edge_weights: Option<BTreeMap<String, f64>>,
}

/// A prior partition for stability scoring.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPriorPartition {
    /// node_id → community_id mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
}

/// A prior partition for [`infer_boundaries`] — mirrors [`sdivi_core::SnapshotPriorPartition`].
/// Kept separate from [`WasmPriorPartition`] to surface struct-shape divergence at compile time.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmSnapshotPriorPartition {
    /// node_id → community_id mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
}

/// Source location of a pattern instance.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPatternLocationInput {
    pub file: String,
    pub start_row: u32,
    pub start_col: u32,
}

/// A single pattern instance for [`compute_pattern_metrics`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPatternInstanceInput {
    pub fingerprint: String,
    pub category: String,
    pub node_id: String,
    #[tsify(optional)]
    pub location: Option<WasmPatternLocationInput>,
}

/// A node in the pattern AST subtree for [`normalize_and_hash`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmNormalizeNode {
    pub kind: String,
    pub children: Vec<WasmNormalizeNode>,
}

/// Per-category threshold override.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdOverrideInput {
    #[tsify(optional)]
    pub pattern_entropy_rate: Option<f64>,
    #[tsify(optional)]
    pub convention_drift_rate: Option<f64>,
    #[tsify(optional)]
    pub coupling_delta_rate: Option<f64>,
    #[tsify(optional)]
    pub boundary_violation_rate: Option<f64>,
    /// ISO-8601 expiry date `"YYYY-MM-DD"`.
    pub expires: String,
}

/// Threshold configuration for [`compute_thresholds_check`].
/// `today` is an ISO-8601 date string `"YYYY-MM-DD"`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdsInput {
    pub pattern_entropy_rate: f64,
    pub convention_drift_rate: f64,
    pub coupling_delta_rate: f64,
    pub boundary_violation_rate: f64,
    #[serde(default)]
    pub overrides: BTreeMap<String, WasmThresholdOverrideInput>,
    /// ISO-8601 date for threshold expiry evaluation (e.g. `"2026-05-01"`).
    pub today: String,
}

/// A single boundary definition.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundaryDefInput {
    pub name: String,
    pub modules: Vec<String>,
    pub allow_imports_from: Vec<String>,
}

/// Boundary specification for [`compute_boundary_violations`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundarySpecInput {
    pub boundaries: Vec<WasmBoundaryDefInput>,
}

/// Output of [`compute_coupling_topology`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmCouplingTopologyResult {
    pub node_count: usize,
    pub edge_count: usize,
    pub density: f64,
    pub cycle_count: usize,
    pub top_hubs: Vec<(String, usize)>,
    pub component_count: usize,
}

/// Output of [`detect_boundaries`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundaryDetectionResult {
    pub cluster_assignments: BTreeMap<String, u32>,
    pub community_count: u32,
    pub modularity: f64,
    /// community_id (as string) → internal edge density.
    pub internal_edge_density: BTreeMap<String, f64>,
    pub historical_stability: f64,
    pub disconnected_components: u32,
}

/// Output of [`compute_boundary_violations`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundaryViolationResult {
    pub violation_count: u32,
    pub violations: Vec<(String, String)>,
}

/// Per-dimension divergence between two snapshots — output of [`compute_delta`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug, Default)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmDivergenceSummary {
    #[tsify(optional)]
    pub pattern_entropy_delta: Option<f64>,
    #[tsify(optional)]
    pub convention_drift_delta: Option<f64>,
    #[tsify(optional)]
    pub coupling_delta: Option<f64>,
    #[tsify(optional)]
    pub community_count_delta: Option<i64>,
    #[tsify(optional)]
    pub boundary_violation_delta: Option<i64>,
    /// Per-category entropy delta; `None` on the first-snapshot path.
    #[serde(default)]
    #[tsify(optional)]
    pub pattern_entropy_per_category_delta: Option<BTreeMap<String, f64>>,
    /// Per-category convention-drift delta; `None` on the first-snapshot path.
    #[serde(default)]
    #[tsify(optional)]
    pub convention_drift_per_category_delta: Option<BTreeMap<String, f64>>,
}

/// Pattern metrics output (also used as a snapshot sub-field).
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmPatternMetricsResult {
    /// Shannon entropy per category.
    pub entropy_per_category: BTreeMap<String, f64>,
    /// Sum of per-category entropies.
    pub total_entropy: f64,
    /// Average `distinct / total` across all categories.
    pub convention_drift: f64,
    /// Per-category `distinct / total` before averaging.
    #[serde(default)]
    pub convention_drift_per_category: BTreeMap<String, f64>,
}

/// A single threshold breach.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdBreachInfo {
    /// Name of the dimension that exceeded its limit.
    pub dimension: String,
    /// Category name for per-category breaches; absent for aggregate breaches.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub category: Option<String>,
    /// Observed delta value.
    pub actual: f64,
    /// The limit that was exceeded.
    pub limit: f64,
}

/// Diagnostic info for one entry in `WasmThresholdCheckResult::applied_overrides`.
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmAppliedOverrideInfo {
    /// Whether the override was active (not expired) at evaluation time.
    pub active: bool,
    /// Expiry date as `"YYYY-MM-DD"`.
    pub expires: String,
    /// Human-readable explanation when inactive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional)]
    pub expired_reason: Option<String>,
}

/// Output of [`compute_thresholds_check`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmThresholdCheckResult {
    /// `true` when at least one threshold was exceeded.
    pub breached: bool,
    /// Per-dimension details for each exceeded threshold.
    pub breaches: Vec<WasmThresholdBreachInfo>,
    /// Diagnostic map of every override entry with `active` flag and `expires` date.
    pub applied_overrides: BTreeMap<String, WasmAppliedOverrideInfo>,
}

/// A proposed boundary from [`infer_boundaries`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundaryProposal {
    pub community_id: u32,
    pub stable_snapshots: u32,
    pub node_ids: Vec<String>,
}

/// Output of [`infer_boundaries`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmBoundaryInferenceResult {
    pub proposals: Vec<WasmBoundaryProposal>,
    pub partition_count: usize,
}

/// Output of [`compute_trend`].
#[derive(Tsify, Serialize, Deserialize, Clone, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WasmTrendResult {
    pub snapshot_count: usize,
    #[tsify(optional)]
    pub pattern_entropy_slope: Option<f64>,
    #[tsify(optional)]
    pub convention_drift_slope: Option<f64>,
    #[tsify(optional)]
    pub coupling_slope: Option<f64>,
    #[tsify(optional)]
    pub community_count_slope: Option<f64>,
}

// ── assemble_snapshot input (moved to assemble_types.rs) ─────────────────────
pub use crate::assemble_types::WasmAssembleSnapshotInput;
