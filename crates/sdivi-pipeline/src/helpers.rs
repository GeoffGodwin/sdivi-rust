//! Private helpers shared within the pipeline crate.

use std::collections::BTreeMap;

use sdivi_config::BoundarySpec;
use sdivi_core::input::{
    BoundaryDefInput, BoundarySpecInput, DependencyGraphInput, EdgeInput, NodeInput,
};
use sdivi_detection::partition::LeidenPartition;
use sdivi_graph::dependency_graph::DependencyGraph;
use sdivi_snapshot::change_coupling::ChangeCouplingResult;

/// Builds a `(min_idx, max_idx) → weight` edge-weight map for weighted Leiden.
///
/// Weight = `1.0 + frequency`. Only pairs whose both endpoints exist in `dg`
/// produce entries.
pub(crate) fn build_edge_weight_map(
    dg: &DependencyGraph,
    ccr: &ChangeCouplingResult,
) -> BTreeMap<(usize, usize), f64> {
    let mut map = BTreeMap::new();
    for pair in &ccr.pairs {
        let sp = std::path::Path::new(&pair.source);
        let tp = std::path::Path::new(&pair.target);
        if let (Some(si), Some(ti)) = (dg.node_for_path(sp), dg.node_for_path(tp)) {
            let key = if si < ti { (si, ti) } else { (ti, si) };
            map.insert(key, 1.0 + pair.frequency);
        }
    }
    map
}

/// Builds a path→community mapping from a `DependencyGraph` and `LeidenPartition`.
///
/// Nodes with no valid UTF-8 path are silently dropped.
pub(crate) fn compute_path_partition(
    dg: &DependencyGraph,
    partition: &LeidenPartition,
) -> BTreeMap<String, u32> {
    let mut map = BTreeMap::new();
    for (&node_idx, &comm_id) in &partition.assignments {
        if let Some(path) = dg.node_path(node_idx) {
            if let Some(s) = path.to_str() {
                map.insert(s.to_string(), comm_id as u32);
            }
        }
    }
    map
}

/// Converts a [`DependencyGraph`] to a [`DependencyGraphInput`] for pure-compute calls.
///
/// Node IDs use forward-slash-normalised repo-relative paths.  Language is left empty
/// because boundary violation detection does not use it.
pub(crate) fn graph_to_boundary_input(dg: &DependencyGraph) -> DependencyGraphInput {
    let n = dg.node_count();
    let ids: Vec<String> = (0..n)
        .map(|i| {
            dg.node_path(i)
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default()
        })
        .collect();
    let nodes: Vec<NodeInput> = ids
        .iter()
        .map(|id| NodeInput {
            id: id.clone(),
            path: id.clone(),
            language: String::new(),
        })
        .collect();
    let edges: Vec<EdgeInput> = dg
        .edges_as_pairs()
        .into_iter()
        .filter_map(|(f, t)| {
            let src = ids.get(f)?.clone();
            let tgt = ids.get(t)?.clone();
            Some(EdgeInput {
                source: src,
                target: tgt,
            })
        })
        .collect();
    DependencyGraphInput { nodes, edges }
}

/// Converts a [`BoundarySpec`] to a [`BoundarySpecInput`] for pure-compute calls.
pub(crate) fn spec_to_boundary_input(spec: &BoundarySpec) -> BoundarySpecInput {
    BoundarySpecInput {
        boundaries: spec
            .boundaries
            .iter()
            .map(|b| BoundaryDefInput {
                name: b.name.clone(),
                modules: b.modules.clone(),
                allow_imports_from: b.allow_imports_from.clone(),
            })
            .collect(),
    }
}
