//! [`compute_coupling_topology`] — dependency graph metrics from input structs.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::AnalysisError;
use crate::input::{validate_node_id, DependencyGraphInput};

/// Coupling topology metrics derived from a [`DependencyGraphInput`].
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::DependencyGraphInput;
/// use sdivi_core::compute::coupling::compute_coupling_topology;
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let result = compute_coupling_topology(&g).unwrap();
/// assert_eq!(result.node_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CouplingTopologyResult {
    /// Total node count.
    pub node_count: usize,
    /// Total directed edge count.
    pub edge_count: usize,
    /// Graph density: `edge_count / (node_count * (node_count - 1))`.
    pub density: f64,
    /// Number of directed cycles.
    pub cycle_count: usize,
    /// Top nodes by out-degree: `(node_id, out_degree)`.
    pub top_hubs: Vec<(String, usize)>,
    /// Number of weakly connected components.
    pub component_count: usize,
}

/// Computes dependency graph metrics from a [`DependencyGraphInput`].
///
/// Validates all node IDs before building the graph.
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] if any node ID violates the
/// canonicalization rules.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{DependencyGraphInput, NodeInput, EdgeInput};
/// use sdivi_core::compute::coupling::compute_coupling_topology;
///
/// let g = DependencyGraphInput {
///     nodes: vec![
///         NodeInput { id: "src/lib.rs".to_string(), path: "src/lib.rs".to_string(), language: "rust".to_string() },
///         NodeInput { id: "src/models.rs".to_string(), path: "src/models.rs".to_string(), language: "rust".to_string() },
///     ],
///     edges: vec![EdgeInput { source: "src/lib.rs".to_string(), target: "src/models.rs".to_string() }],
/// };
/// let result = compute_coupling_topology(&g).unwrap();
/// assert_eq!(result.node_count, 2);
/// assert_eq!(result.edge_count, 1);
/// ```
pub fn compute_coupling_topology(
    graph: &DependencyGraphInput,
) -> Result<CouplingTopologyResult, AnalysisError> {
    for node in &graph.nodes {
        validate_node_id(&node.id)?;
    }

    let node_paths: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let id_to_idx: BTreeMap<&str, usize> = graph
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.id.as_str(), i))
        .collect();

    let raw_edges: Vec<(usize, usize)> = graph
        .edges
        .iter()
        .filter_map(|e| {
            let from = *id_to_idx.get(e.source.as_str())?;
            let to = *id_to_idx.get(e.target.as_str())?;
            if from != to {
                Some((from, to))
            } else {
                None
            }
        })
        .collect();

    let dg =
        sdivi_graph::dependency_graph::build_dependency_graph_from_edges(&node_paths, &raw_edges);
    let metrics = sdivi_graph::metrics::compute_metrics(&dg);

    let top_hubs: Vec<(String, usize)> = metrics
        .top_hubs
        .into_iter()
        .map(|(path, deg)| (path.to_string_lossy().into_owned(), deg))
        .collect();

    Ok(CouplingTopologyResult {
        node_count: metrics.node_count,
        edge_count: metrics.edge_count,
        density: metrics.density,
        cycle_count: metrics.cycle_count,
        top_hubs,
        component_count: metrics.component_count,
    })
}
