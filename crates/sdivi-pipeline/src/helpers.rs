//! Private helpers shared within the pipeline crate.

use std::collections::BTreeMap;

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
