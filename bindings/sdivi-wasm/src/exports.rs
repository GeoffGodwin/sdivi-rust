//! `#[wasm_bindgen]` exported functions — wraps every `sdivi_core::compute_*` fn.
//!
//! **Conversion strategy**: wrapper types and sdivi-core types share the same
//! serde field names, so a serde_json round-trip converts between them without
//! explicit From impls.  The exception is `assemble_snapshot`, which builds
//! internal types (LeidenPartition, GraphMetrics, PatternCatalog) by hand.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::types::*;
use crate::weight_keys::parse_wasm_edge_weights;

// ── Conversion helpers ────────────────────────────────────────────────────────

fn err(e: impl std::fmt::Display) -> JsError {
    JsError::new(&e.to_string())
}

/// Round-trip a wrapper type to a sdivi-core type via serde_json.
fn to_core<W: Serialize, C: for<'de> Deserialize<'de>>(w: W) -> Result<C, JsError> {
    let v = serde_json::to_value(w).map_err(err)?;
    serde_json::from_value(v).map_err(err)
}

/// Serialize a sdivi-core result to a wrapper type via serde_json.
fn from_core<C: Serialize, W: for<'de> Deserialize<'de>>(c: C) -> Result<W, JsError> {
    let v = serde_json::to_value(c).map_err(err)?;
    serde_json::from_value(v).map_err(err)
}

// ── Simple compute exports ────────────────────────────────────────────────────

/// Compute dependency graph coupling metrics.
#[wasm_bindgen]
pub fn compute_coupling_topology(
    graph: WasmDependencyGraphInput,
) -> Result<WasmCouplingTopologyResult, JsError> {
    let g = to_core(graph)?;
    let result = sdivi_core::compute_coupling_topology(&g).map_err(err)?;
    from_core(result)
}

/// Run Leiden community detection and return cluster assignments + stability.
///
/// When `cfg.edge_weights` is `Some`, runs weighted Leiden. Keys must be
/// `"source:target"` strings (first colon separates source from target, so
/// node IDs that themselves contain colons are fully supported). Weights must
/// be `>= 0.0` and finite; edges absent from the graph are silently ignored.
#[wasm_bindgen]
pub fn detect_boundaries(
    graph: WasmDependencyGraphInput,
    cfg: WasmLeidenConfigInput,
    prior: Vec<WasmPriorPartition>,
) -> Result<WasmBoundaryDetectionResult, JsError> {
    let g = to_core(graph)?;
    // Extract edge_weights before serde round-trip (WASM uses "src:tgt" colon keys;
    // native LeidenConfigInput uses NUL-separated keys via edge_weight_key).
    let wasm_weights = cfg.edge_weights.clone();
    let cfg_no_weights = WasmLeidenConfigInput {
        edge_weights: None,
        ..cfg
    };
    let mut c: sdivi_core::LeidenConfigInput = to_core(cfg_no_weights)?;
    if let Some(ew) = wasm_weights {
        c.edge_weights = Some(parse_wasm_edge_weights(ew).map_err(|e| JsError::new(&e))?);
    }
    let p: Vec<sdivi_core::PriorPartition> =
        prior.into_iter().map(to_core).collect::<Result<_, _>>()?;
    let result = sdivi_core::detect_boundaries(&g, &c, &p).map_err(err)?;
    from_core(result)
}

/// Detect cross-boundary dependency violations against a boundary spec.
#[wasm_bindgen]
pub fn compute_boundary_violations(
    graph: WasmDependencyGraphInput,
    spec: WasmBoundarySpecInput,
) -> Result<WasmBoundaryViolationResult, JsError> {
    let g = to_core(graph)?;
    let s = to_core(spec)?;
    let result = sdivi_core::compute_boundary_violations(&g, &s).map_err(err)?;
    from_core(result)
}

/// Compute Shannon entropy and convention drift from pattern instances.
#[wasm_bindgen]
pub fn compute_pattern_metrics(
    patterns: Vec<WasmPatternInstanceInput>,
) -> Result<WasmPatternMetricsResult, JsError> {
    let p: Vec<sdivi_core::PatternInstanceInput> = patterns
        .into_iter()
        .map(to_core)
        .collect::<Result<_, _>>()?;
    let result = sdivi_core::compute_pattern_metrics(&p);
    from_core(result)
}

/// Check whether any dimension of a divergence summary exceeds thresholds.
#[wasm_bindgen]
pub fn compute_thresholds_check(
    summary: WasmDivergenceSummary,
    cfg: WasmThresholdsInput,
) -> Result<WasmThresholdCheckResult, JsError> {
    let s = to_core(summary)?;
    let c = to_core(cfg)?;
    let result = sdivi_core::compute_thresholds_check(&s, &c);
    from_core(result)
}

/// Compute per-dimension divergence between two snapshots (JSON objects).
#[wasm_bindgen]
pub fn compute_delta(prev: JsValue, curr: JsValue) -> Result<WasmDivergenceSummary, JsError> {
    let p: sdivi_core::Snapshot = serde_wasm_bindgen::from_value(prev).map_err(err)?;
    let c: sdivi_core::Snapshot = serde_wasm_bindgen::from_value(curr).map_err(err)?;
    let result = sdivi_core::compute_delta(&p, &c);
    from_core(result)
}

/// Compute trend statistics over an array of snapshot JSON objects.
#[wasm_bindgen]
pub fn compute_trend(snapshots: JsValue, last_n: Option<u32>) -> Result<WasmTrendResult, JsError> {
    let snaps: Vec<sdivi_core::Snapshot> =
        serde_wasm_bindgen::from_value(snapshots).map_err(err)?;
    let n = last_n.map(|x| x as usize);
    let result = sdivi_core::compute_trend(&snaps, n);
    from_core(result)
}

/// Infer boundary proposals from a sequence of prior partitions.
#[wasm_bindgen]
pub fn infer_boundaries(
    prior_partitions: Vec<WasmSnapshotPriorPartition>,
    stability_threshold: u32,
) -> Result<WasmBoundaryInferenceResult, JsError> {
    let partitions: Vec<sdivi_core::SnapshotPriorPartition> = prior_partitions
        .into_iter()
        .map(|p| sdivi_core::SnapshotPriorPartition {
            cluster_assignments: p.cluster_assignments,
        })
        .collect();
    let result = sdivi_core::infer_boundaries(&partitions, stability_threshold);
    from_core(result)
}

/// Compute a canonical blake3 fingerprint for a pattern AST node.
///
/// Returns a 64-character lowercase hex string that is byte-identical to the
/// fingerprint produced by the native Rust pipeline for the same input.
#[wasm_bindgen]
pub fn normalize_and_hash(
    node_kind: &str,
    children: Vec<WasmNormalizeNode>,
) -> Result<String, JsError> {
    let c: Vec<sdivi_core::NormalizeNode> = children
        .into_iter()
        .map(to_core)
        .collect::<Result<_, _>>()?;
    Ok(sdivi_core::normalize_and_hash(node_kind, &c))
}

// ── assemble_snapshot ────────────────────────────────────────────────────────

/// Assemble a Snapshot from compute-function outputs.
///
/// Returns a snapshot JSON object that can be passed to `compute_delta` or
/// stored in `.sdivi/snapshots/`.
#[wasm_bindgen]
pub fn assemble_snapshot(input: WasmAssembleSnapshotInput) -> Result<JsValue, JsError> {
    let graph = build_graph_metrics(&input);
    let partition = build_leiden_partition(&input)?;
    let catalog = build_pattern_catalog(&input.pattern_instances)?;
    let pm: sdivi_core::PatternMetricsResult = to_core(input.pattern_metrics)?;

    let change_coupling: Option<sdivi_core::ChangeCouplingResult> =
        input.change_coupling.map(to_core).transpose()?;

    let mut snap = sdivi_core::assemble_snapshot(
        graph,
        partition,
        catalog,
        pm,
        None,
        &input.timestamp,
        input.commit.as_deref(),
        change_coupling,
        0,
    );

    if let Some(count) = input.boundary_count {
        snap.intent_divergence = Some(sdivi_core::IntentDivergenceInfo {
            boundary_count: count as usize,
            violation_count: input.violation_count.unwrap_or(0),
        });
    }

    serde_wasm_bindgen::to_value(&snap).map_err(err)
}

fn build_graph_metrics(input: &WasmAssembleSnapshotInput) -> sdivi_core::GraphMetrics {
    sdivi_core::GraphMetrics {
        node_count: input.node_count,
        edge_count: input.edge_count,
        density: input.density,
        cycle_count: input.cycle_count,
        // top_hubs stores PathBuf internally; TS consumers see [string, number][].
        // If GraphMetrics.top_hubs changes element type, update this conversion.
        top_hubs: input
            .top_hubs
            .iter()
            .map(|(id, deg)| (PathBuf::from(id), *deg))
            .collect(),
        component_count: input.component_count,
    }
}

fn build_leiden_partition(
    input: &WasmAssembleSnapshotInput,
) -> Result<sdivi_core::LeidenPartition, JsError> {
    let id_to_idx: BTreeMap<&str, usize> = input
        .node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();

    let assignments: BTreeMap<usize, usize> = input
        .cluster_assignments
        .iter()
        .filter_map(|(id, &comm)| id_to_idx.get(id.as_str()).map(|&idx| (idx, comm as usize)))
        .collect();

    let stability: BTreeMap<usize, f64> = input
        .internal_edge_density
        .iter()
        .map(|(comm_str, &density)| {
            let comm: usize = comm_str.parse().map_err(err)?;
            Ok((comm, density))
        })
        .collect::<Result<_, JsError>>()?;

    Ok(sdivi_core::LeidenPartition {
        assignments,
        stability,
        modularity: input.modularity,
        seed: input.leiden_seed.unwrap_or(42),
    })
}

fn build_pattern_catalog(
    instances: &[WasmPatternInstanceInput],
) -> Result<sdivi_core::PatternCatalog, JsError> {
    let mut catalog = sdivi_core::PatternCatalog::default();
    for inst in instances {
        let fp = sdivi_core::PatternFingerprint::from_hex(&inst.fingerprint).ok_or_else(|| {
            JsError::new(&format!("invalid fingerprint hex: {}", inst.fingerprint))
        })?;
        let stats = catalog
            .entries
            .entry(inst.category.clone())
            .or_default()
            .entry(fp)
            .or_insert(sdivi_core::PatternStats {
                count: 0,
                locations: vec![],
            });
        stats.count += 1;
    }
    Ok(catalog)
}
