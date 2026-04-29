//! [`DependencyGraph`] construction from [`FeatureRecord`] slices.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use petgraph::Graph;
use petgraph::graph::NodeIndex;
use sdi_parsing::feature_record::FeatureRecord;
use thiserror::Error;
use tracing::debug;

/// Errors that can occur during graph construction.
#[derive(Debug, Error)]
pub enum GraphError {
    /// No source files were provided to the builder.
    #[error("no source records provided")]
    Empty,
}

/// Directed dependency graph with one node per source file.
///
/// Nodes carry the file path (relative to the repository root).
/// Edges are directed from the importing file to the imported file.
/// Only intra-repository imports are resolved; external imports are silently
/// dropped and logged at `DEBUG` level.
///
/// # Examples
///
/// ```rust
/// use sdi_graph::dependency_graph::{DependencyGraph, build_dependency_graph};
/// use sdi_parsing::feature_record::FeatureRecord;
/// use std::path::PathBuf;
///
/// let records = vec![
///     FeatureRecord {
///         path: PathBuf::from("src/lib.rs"),
///         language: "rust".to_string(),
///         imports: vec!["crate::models".to_string()],
///         exports: vec![],
///         signatures: vec![],
///         pattern_hints: vec![],
///     },
///     FeatureRecord {
///         path: PathBuf::from("src/models.rs"),
///         language: "rust".to_string(),
///         imports: vec![],
///         exports: vec!["Record".to_string()],
///         signatures: vec![],
///         pattern_hints: vec![],
///     },
/// ];
/// let dg = build_dependency_graph(&records);
/// assert_eq!(dg.node_count(), 2);
/// assert_eq!(dg.edge_count(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub(crate) graph: Graph<PathBuf, ()>,
    pub(crate) path_to_node: BTreeMap<PathBuf, NodeIndex>,
}

impl DependencyGraph {
    /// Number of nodes (source files) in the graph.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Number of directed edges in the graph.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Returns the file path for node index `idx`.
    ///
    /// Returns `None` if `idx` is out of range.
    pub fn node_path(&self, idx: usize) -> Option<&Path> {
        let ni = NodeIndex::new(idx);
        self.graph.node_weight(ni).map(|p| p.as_path())
    }

    /// Returns the node index for `path`, or `None` if the path is not in the graph.
    pub fn node_for_path(&self, path: &Path) -> Option<usize> {
        self.path_to_node.get(path).map(|ni| ni.index())
    }

    /// Returns all directed edges as `(from_idx, to_idx)` pairs.
    pub fn edges_as_pairs(&self) -> Vec<(usize, usize)> {
        self.graph
            .raw_edges()
            .iter()
            .map(|e| (e.source().index(), e.target().index()))
            .collect()
    }

    /// Returns the neighbors of node `idx` (nodes that `idx` imports from).
    pub fn neighbors(&self, idx: usize) -> Vec<usize> {
        let ni = NodeIndex::new(idx);
        self.graph.neighbors(ni).map(|n| n.index()).collect()
    }
}

/// Builds a [`DependencyGraph`] from a slice of [`FeatureRecord`]s.
///
/// Each record becomes one node. Import strings are resolved against the set
/// of known file paths; unresolvable imports are dropped at `DEBUG` level.
///
/// # Examples
///
/// ```rust
/// use sdi_graph::dependency_graph::build_dependency_graph;
/// use sdi_parsing::feature_record::FeatureRecord;
/// use std::path::PathBuf;
///
/// let records: Vec<FeatureRecord> = vec![];
/// let dg = build_dependency_graph(&records);
/// assert_eq!(dg.node_count(), 0);
/// ```
pub fn build_dependency_graph(records: &[FeatureRecord]) -> DependencyGraph {
    let mut graph: Graph<PathBuf, ()> = Graph::new();
    let mut path_to_node: BTreeMap<PathBuf, NodeIndex> = BTreeMap::new();

    // First pass: add all nodes.
    for record in records {
        let ni = graph.add_node(record.path.clone());
        path_to_node.insert(record.path.clone(), ni);
    }

    // Build stem → node-index lookup for import resolution.
    let stem_map = build_stem_map(&path_to_node);

    // Second pass: resolve imports and add edges.
    for record in records {
        let from_ni = path_to_node[&record.path];
        for import in &record.imports {
            match resolve_import(import, &record.path, &stem_map, &path_to_node) {
                Some(to_ni) if to_ni != from_ni => {
                    // Avoid duplicate edges (petgraph allows them; we don't want them).
                    if !graph.contains_edge(from_ni, to_ni) {
                        graph.add_edge(from_ni, to_ni, ());
                    }
                }
                Some(_) => {} // self-loop: skip
                None => {
                    debug!(%import, path = ?record.path, "unresolved import dropped");
                }
            }
        }
    }

    DependencyGraph { graph, path_to_node }
}

/// Maps file stem (lowercase) → list of NodeIndex values.
fn build_stem_map(
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> BTreeMap<String, Vec<NodeIndex>> {
    let mut map: BTreeMap<String, Vec<NodeIndex>> = BTreeMap::new();
    for (path, &ni) in path_to_node {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            map.entry(stem.to_ascii_lowercase()).or_default().push(ni);
        }
    }
    map
}

/// Attempts to resolve an import string to a known [`NodeIndex`].
///
/// Resolution order:
/// 1. Relative path (`./foo`, `../foo`) → strip prefix, find matching path.
/// 2. Rust local prefix (`crate::X`, `self::X`, `super::X`) → resolve `X` as stem.
/// 3. All other imports → `None` (external; logged at `DEBUG` by caller).
fn resolve_import(
    import: &str,
    from_path: &Path,
    stem_map: &BTreeMap<String, Vec<NodeIndex>>,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Option<NodeIndex> {
    // Strategy 1: Relative path import (JS / TS / Python relative).
    if import.starts_with("./") || import.starts_with("../") {
        return resolve_relative(import, from_path, path_to_node);
    }

    // Strategy 2: Rust local-crate prefix.
    let local = import
        .strip_prefix("crate::")
        .or_else(|| import.strip_prefix("self::"))
        .or_else(|| import.strip_prefix("super::"));

    if let Some(local) = local {
        let first = local.split("::").next()?;
        return resolve_stem(first, stem_map);
    }

    None
}

/// Resolves a relative import path (`./foo` or `../bar`) to a NodeIndex.
fn resolve_relative(
    import: &str,
    from_path: &Path,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Option<NodeIndex> {
    let from_dir = from_path.parent()?;
    let rel = import.trim_start_matches("./").trim_start_matches("../");
    // Try common extensions.
    for ext in &["rs", "py", "ts", "tsx", "js", "go", "java"] {
        let candidate = from_dir.join(format!("{rel}.{ext}"));
        if let Some(&ni) = path_to_node.get(&candidate) {
            return Some(ni);
        }
    }
    // Try as a directory module (e.g., `./foo` → `./foo/mod.rs` or `./foo/index.ts`).
    for index in &["mod.rs", "index.ts", "index.js", "__init__.py"] {
        let candidate = from_dir.join(rel).join(index);
        if let Some(&ni) = path_to_node.get(&candidate) {
            return Some(ni);
        }
    }
    None
}

/// Resolves a module stem to a NodeIndex (unambiguous match only).
fn resolve_stem(
    stem: &str,
    stem_map: &BTreeMap<String, Vec<NodeIndex>>,
) -> Option<NodeIndex> {
    let candidates = stem_map.get(&stem.to_ascii_lowercase())?;
    if candidates.len() == 1 {
        Some(candidates[0])
    } else {
        // Ambiguous: two files share the same stem — cannot resolve safely.
        None
    }
}
