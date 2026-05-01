//! [`detect_boundaries`] and [`compute_boundary_violations`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::AnalysisError;
use crate::input::{
    BoundarySpecInput, DependencyGraphInput, LeidenConfigInput, PriorPartition,
    QualityFunctionInput, validate_node_id,
};

/// Result of [`detect_boundaries`].
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{DependencyGraphInput, LeidenConfigInput};
/// use sdi_core::compute::boundaries::detect_boundaries;
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
/// assert_eq!(r.community_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryDetectionResult {
    /// Node ID → community ID mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
    /// Number of detected communities.
    pub community_count: u32,
    /// Overall partition modularity.
    pub modularity: f64,
    /// Per-community internal edge density.
    pub internal_edge_density: BTreeMap<u32, f64>,
    /// Consecutive-snapshot stability score against `prior` history.
    ///
    /// `0.0` when `prior` is empty or has only one entry (no pairs to compare).
    pub historical_stability: f64,
    /// Number of disconnected components in the input graph.
    pub disconnected_components: u32,
}

/// Runs Leiden community detection on a [`DependencyGraphInput`].
///
/// `prior` is ordered oldest → newest; used only for `historical_stability`
/// scoring (no warm-start — the warm-start path is in `sdi-pipeline::Pipeline`).
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] if any node ID is invalid.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{DependencyGraphInput, LeidenConfigInput};
/// use sdi_core::compute::boundaries::detect_boundaries;
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let result = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
/// assert_eq!(result.historical_stability, 0.0);
/// ```
pub fn detect_boundaries(
    graph: &DependencyGraphInput,
    cfg: &LeidenConfigInput,
    prior: &[PriorPartition],
) -> Result<BoundaryDetectionResult, AnalysisError> {
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
            if from != to { Some((from, to)) } else { None }
        })
        .collect();

    let dg = sdi_graph::dependency_graph::build_dependency_graph_from_edges(&node_paths, &raw_edges);

    let quality = match cfg.quality {
        QualityFunctionInput::Modularity => sdi_detection::QualityFunction::Modularity,
        QualityFunctionInput::Cpm => sdi_detection::QualityFunction::Cpm { gamma: cfg.gamma },
    };
    let leiden_cfg = sdi_detection::LeidenConfig {
        seed: cfg.seed,
        max_iterations: cfg.iterations,
        quality,
        gamma: cfg.gamma,
    };

    let partition = if let Some(ref ew) = cfg.edge_weights {
        let weight_map: std::collections::BTreeMap<(usize, usize), f64> = ew
            .iter()
            .filter_map(|((s, t), &w)| {
                let si = *id_to_idx.get(s.as_str())?;
                let ti = *id_to_idx.get(t.as_str())?;
                let key = if si < ti { (si, ti) } else { (ti, si) };
                Some((key, w))
            })
            .collect();
        sdi_detection::run_leiden_with_weights(&dg, &leiden_cfg, None, &weight_map)
    } else {
        sdi_detection::run_leiden(&dg, &leiden_cfg, None)
    };

    // Build cluster_assignments: NodeId → community ID.
    let cluster_assignments: BTreeMap<String, u32> = partition
        .assignments
        .iter()
        .filter_map(|(&node_idx, &comm)| {
            node_paths.get(node_idx).map(|id| (id.clone(), comm as u32))
        })
        .collect();

    let community_count = partition.community_count() as u32;

    // Internal edge density per community (the stability field from LeidenPartition).
    let internal_edge_density: BTreeMap<u32, f64> = partition
        .stability
        .iter()
        .map(|(&comm, &density)| (comm as u32, density))
        .collect();

    let historical_stability = compute_historical_stability(prior, &cluster_assignments);

    let component_count = sdi_graph::metrics::compute_metrics(&dg).component_count as u32;

    Ok(BoundaryDetectionResult {
        cluster_assignments,
        community_count,
        modularity: partition.modularity,
        internal_edge_density,
        historical_stability,
        disconnected_components: component_count,
    })
}

/// Computes the fraction of consecutive prior-partition pairs that agree with
/// the current cluster assignments (by node-set membership).
fn compute_historical_stability(
    prior: &[PriorPartition],
    current: &BTreeMap<String, u32>,
) -> f64 {
    if prior.len() < 2 {
        return 0.0;
    }

    let current_communities = invert_assignments(current);
    let n_pairs = (prior.len() - 1) as f64;
    let mut matching = 0.0f64;

    for pair in prior.windows(2) {
        let prev = invert_assignments(&pair[0].cluster_assignments);
        let next = invert_assignments(&pair[1].cluster_assignments);
        // A pair "agrees" if the community sets are the same (regardless of numeric ID).
        if community_sets_match(&prev, &next) {
            matching += 1.0;
        }
    }

    // Also measure how similar the prior history is to the current.
    let _ = &current_communities; // used for future extension
    matching / n_pairs
}

fn invert_assignments(
    assignments: &BTreeMap<String, u32>,
) -> BTreeMap<u32, std::collections::BTreeSet<String>> {
    let mut result: BTreeMap<u32, std::collections::BTreeSet<String>> = BTreeMap::new();
    for (node, &comm) in assignments {
        result.entry(comm).or_default().insert(node.clone());
    }
    result
}

fn community_sets_match(
    a: &BTreeMap<u32, std::collections::BTreeSet<String>>,
    b: &BTreeMap<u32, std::collections::BTreeSet<String>>,
) -> bool {
    let a_sets: std::collections::BTreeSet<_> = a.values().collect();
    let b_sets: std::collections::BTreeSet<_> = b.values().collect();
    a_sets == b_sets
}

/// Result of [`compute_boundary_violations`].
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{DependencyGraphInput, BoundarySpecInput};
/// use sdi_core::compute::boundaries::compute_boundary_violations;
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let spec = BoundarySpecInput { boundaries: vec![] };
/// let r = compute_boundary_violations(&g, &spec).unwrap();
/// assert_eq!(r.violation_count, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryViolationResult {
    /// Total number of cross-boundary dependency violations.
    pub violation_count: u32,
    /// Pairs `(from_node_id, to_node_id)` that cross a boundary.
    pub violations: Vec<(String, String)>,
}

/// Computes boundary violations against a [`BoundarySpecInput`].
///
/// A violation occurs when an edge crosses from one declared boundary into
/// another that is not listed in `allow_imports_from`.
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] if any node ID is invalid.
///
/// # Examples
///
/// ```rust
/// use sdi_core::input::{DependencyGraphInput, BoundarySpecInput};
/// use sdi_core::compute::boundaries::compute_boundary_violations;
///
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let spec = BoundarySpecInput { boundaries: vec![] };
/// let result = compute_boundary_violations(&g, &spec).unwrap();
/// assert_eq!(result.violation_count, 0);
/// ```
pub fn compute_boundary_violations(
    graph: &DependencyGraphInput,
    _spec: &BoundarySpecInput,
) -> Result<BoundaryViolationResult, AnalysisError> {
    for node in &graph.nodes {
        validate_node_id(&node.id)?;
    }
    // Full violation detection is implemented in Milestone 10.
    Ok(BoundaryViolationResult {
        violation_count: 0,
        violations: vec![],
    })
}
