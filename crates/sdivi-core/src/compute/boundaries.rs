//! [`detect_boundaries`] and [`compute_boundary_violations`].

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::AnalysisError;
use crate::input::{
    split_edge_weight_key, validate_node_id, BoundarySpecInput, DependencyGraphInput,
    LeidenConfigInput, PriorPartition, QualityFunctionInput,
};

/// Result of [`detect_boundaries`].
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{DependencyGraphInput, LeidenConfigInput};
/// use sdivi_core::compute::boundaries::detect_boundaries;
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
/// scoring (no warm-start — the warm-start path is in `sdivi-pipeline::Pipeline`).
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] if any node ID is invalid.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{DependencyGraphInput, LeidenConfigInput};
/// use sdivi_core::compute::boundaries::detect_boundaries;
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
            if from != to {
                Some((from, to))
            } else {
                None
            }
        })
        .collect();

    let dg =
        sdivi_graph::dependency_graph::build_dependency_graph_from_edges(&node_paths, &raw_edges);

    let quality = match cfg.quality {
        QualityFunctionInput::Modularity => sdivi_detection::QualityFunction::Modularity,
        QualityFunctionInput::Cpm => sdivi_detection::QualityFunction::Cpm { gamma: cfg.gamma },
    };
    let leiden_cfg = sdivi_detection::LeidenConfig {
        seed: cfg.seed,
        max_iterations: cfg.iterations,
        quality,
        gamma: cfg.gamma,
    };

    let partition = if let Some(ref ew) = cfg.edge_weights {
        let weight_map: std::collections::BTreeMap<(usize, usize), f64> = ew
            .iter()
            .filter_map(|(key, &w)| {
                let (s, t) = split_edge_weight_key(key)?;
                let si = *id_to_idx.get(s)?;
                let ti = *id_to_idx.get(t)?;
                let ordered = if si < ti { (si, ti) } else { (ti, si) };
                Some((ordered, w))
            })
            .collect();
        sdivi_detection::run_leiden_with_weights(&dg, &leiden_cfg, None, &weight_map)
    } else {
        sdivi_detection::run_leiden(&dg, &leiden_cfg, None)
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

    let historical_stability = super::stability::compute_historical_stability(prior);

    let component_count = sdivi_graph::metrics::compute_metrics(&dg).component_count as u32;

    Ok(BoundaryDetectionResult {
        cluster_assignments,
        community_count,
        modularity: partition.modularity,
        internal_edge_density,
        historical_stability,
        disconnected_components: component_count,
    })
}

/// Result of [`compute_boundary_violations`].
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{DependencyGraphInput, BoundarySpecInput};
/// use sdivi_core::compute::boundaries::compute_boundary_violations;
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
/// A violation occurs when an edge `(from, to)` crosses from one declared boundary
/// into another AND `to`'s boundary name is absent from `from`-boundary's
/// `allow_imports_from` list.  Edges where either endpoint is unscoped (matches
/// no boundary glob) are silently skipped — unscoped is not a violation.
///
/// Nodes matching multiple boundary globs are assigned the **most specific** match
/// (longest glob string by character length; ties broken by ascending boundary name).
///
/// `allow_imports_from` is NOT transitive: if `a` allows `b` and `b` allows `c`,
/// an `a → c` edge is still a violation unless `a` explicitly lists `c`.
///
/// # Errors
///
/// Returns [`AnalysisError::InvalidNodeId`] if any node ID is invalid.
/// Returns [`AnalysisError::InvalidConfig`] if a boundary glob fails to compile.
///
/// # Examples
///
/// ```rust
/// use sdivi_core::input::{
///     BoundaryDefInput, BoundarySpecInput, DependencyGraphInput, EdgeInput, NodeInput,
/// };
/// use sdivi_core::compute::boundaries::compute_boundary_violations;
///
/// // Empty spec → zero violations regardless of graph content.
/// let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
/// let r = compute_boundary_violations(&g, &BoundarySpecInput { boundaries: vec![] }).unwrap();
/// assert_eq!(r.violation_count, 0);
///
/// // db → api: db has no allow_imports_from → one violation.
/// let g2 = DependencyGraphInput {
///     nodes: vec![
///         NodeInput { id: "crates/db/f.rs".into(), path: "crates/db/f.rs".into(), language: "rust".into() },
///         NodeInput { id: "crates/api/b.rs".into(), path: "crates/api/b.rs".into(), language: "rust".into() },
///     ],
///     edges: vec![EdgeInput { source: "crates/db/f.rs".into(), target: "crates/api/b.rs".into() }],
/// };
/// let spec2 = BoundarySpecInput { boundaries: vec![
///     BoundaryDefInput { name: "api".into(), modules: vec!["crates/api/**".into()], allow_imports_from: vec!["db".into()] },
///     BoundaryDefInput { name: "db".into(), modules: vec!["crates/db/**".into()], allow_imports_from: vec![] },
/// ]};
/// assert_eq!(compute_boundary_violations(&g2, &spec2).unwrap().violation_count, 1);
/// ```
pub fn compute_boundary_violations(
    graph: &DependencyGraphInput,
    spec: &BoundarySpecInput,
) -> Result<BoundaryViolationResult, AnalysisError> {
    for node in &graph.nodes {
        validate_node_id(&node.id)?;
    }

    if spec.boundaries.is_empty() {
        return Ok(BoundaryViolationResult {
            violation_count: 0,
            violations: vec![],
        });
    }

    let compiled = super::violation::compile_boundaries(spec)?;

    let node_boundaries: BTreeMap<&str, &str> = graph
        .nodes
        .iter()
        .filter_map(|n| {
            super::violation::match_boundary(&n.id, &compiled).map(|b| (n.id.as_str(), b))
        })
        .collect();

    let allow_map: BTreeMap<&str, &[String]> = compiled
        .iter()
        .map(|cb| (cb.name.as_str(), cb.allow_imports_from.as_slice()))
        .collect();

    let mut violations: Vec<(String, String)> = Vec::new();

    for edge in &graph.edges {
        let from_b = match node_boundaries.get(edge.source.as_str()) {
            Some(&b) => b,
            None => continue,
        };
        let to_b = match node_boundaries.get(edge.target.as_str()) {
            Some(&b) => b,
            None => continue,
        };
        if from_b == to_b {
            continue;
        }
        let allowed = allow_map
            .get(from_b)
            .is_some_and(|list| list.iter().any(|a| a == to_b));
        if !allowed {
            violations.push((edge.source.clone(), edge.target.clone()));
        }
    }

    violations.sort();
    let violation_count = violations.len() as u32;
    Ok(BoundaryViolationResult {
        violation_count,
        violations,
    })
}
