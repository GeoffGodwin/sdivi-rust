//! [`DependencyGraph`] construction from raw edges or `FeatureRecord` slices.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use petgraph::graph::NodeIndex;
use petgraph::Graph;
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
///
/// # Examples
///
/// ```rust
/// use sdivi_graph::dependency_graph::{DependencyGraph, build_dependency_graph_from_edges};
///
/// let dg = build_dependency_graph_from_edges(
///     &["src/lib.rs".to_string(), "src/models.rs".to_string()],
///     &[(0, 1)],
/// );
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
    pub fn node_path(&self, idx: usize) -> Option<&Path> {
        let ni = NodeIndex::new(idx);
        self.graph.node_weight(ni).map(|p| p.as_path())
    }

    /// Returns the node index for `path`, or `None` if not present.
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

/// Builds a [`DependencyGraph`] from node paths and raw `(from, to)` edge pairs.
///
/// Each entry in `node_paths` becomes one node; edges reference nodes by
/// 0-based index into `node_paths`.  Out-of-range edge indices and self-loops
/// are silently dropped.  Duplicate edges are deduplicated.
///
/// This constructor is always available (no `pipeline-records` feature needed).
///
/// # Examples
///
/// ```rust
/// use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
///
/// let dg = build_dependency_graph_from_edges(
///     &["src/lib.rs".to_string(), "src/models.rs".to_string()],
///     &[(0, 1)],
/// );
/// assert_eq!(dg.node_count(), 2);
/// assert_eq!(dg.edge_count(), 1);
/// ```
pub fn build_dependency_graph_from_edges(
    node_paths: &[String],
    edges: &[(usize, usize)],
) -> DependencyGraph {
    let mut graph: Graph<PathBuf, ()> = Graph::new();
    let mut path_to_node: BTreeMap<PathBuf, NodeIndex> = BTreeMap::new();

    for path_str in node_paths {
        let path = PathBuf::from(path_str);
        let ni = graph.add_node(path.clone());
        path_to_node.insert(path, ni);
    }

    let node_count = node_paths.len();
    for &(from, to) in edges {
        if from >= node_count || to >= node_count || from == to {
            continue;
        }
        let from_ni = NodeIndex::new(from);
        let to_ni = NodeIndex::new(to);
        if !graph.contains_edge(from_ni, to_ni) {
            graph.add_edge(from_ni, to_ni, ());
        }
    }

    DependencyGraph {
        graph,
        path_to_node,
    }
}

/// Builds a [`DependencyGraph`] from a slice of `FeatureRecord`s.
///
/// Import strings are resolved against the set of known file paths; unresolvable
/// imports (external packages, missing files) are dropped at `DEBUG` level.
/// Go module-path imports are not resolved because the module prefix requires
/// explicit supply — use [`build_dependency_graph_with_go_module`] from the
/// pipeline layer when `go.mod` is available.
///
/// # Examples
///
/// ```rust
/// use sdivi_graph::dependency_graph::build_dependency_graph;
/// use sdivi_parsing::feature_record::FeatureRecord;
/// use std::path::PathBuf;
///
/// let records: Vec<FeatureRecord> = vec![];
/// let dg = build_dependency_graph(&records);
/// assert_eq!(dg.node_count(), 0);
/// ```
#[cfg(feature = "pipeline-records")]
pub fn build_dependency_graph(
    records: &[sdivi_parsing::feature_record::FeatureRecord],
) -> DependencyGraph {
    build_dependency_graph_with_tsconfig(records, None, None)
}

/// Builds a [`DependencyGraph`] with an explicit Go module prefix for resolution.
///
/// `go_module` should be the module path from `go.mod` (e.g.
/// `"github.com/myorg/myapp"`). Pass `None` to treat all Go imports as
/// external (same as [`build_dependency_graph`]).
///
/// The pipeline layer reads `go.mod` from the repository root and calls this
/// function to enable internal Go package edges.
///
/// # Examples
///
/// ```rust
/// use sdivi_graph::dependency_graph::build_dependency_graph_with_go_module;
/// use sdivi_parsing::feature_record::FeatureRecord;
///
/// let records: Vec<FeatureRecord> = vec![];
/// let dg = build_dependency_graph_with_go_module(&records, Some("example.com/myapp"));
/// assert_eq!(dg.node_count(), 0);
/// ```
#[cfg(feature = "pipeline-records")]
pub fn build_dependency_graph_with_go_module(
    records: &[sdivi_parsing::feature_record::FeatureRecord],
    go_module: Option<&str>,
) -> DependencyGraph {
    build_dependency_graph_with_tsconfig(records, go_module, None)
}

/// Builds a [`DependencyGraph`] with Go module prefix and tsconfig path aliases.
///
/// Combines M26 Go-module resolution with M27 tsconfig path-alias resolution.
/// `tsconfig` is parsed by the pipeline from `tsconfig.json` / `jsconfig.json`
/// at the repository root and passed here; `None` disables alias resolution.
///
/// # Examples
///
/// ```rust
/// use sdivi_graph::dependency_graph::build_dependency_graph_with_tsconfig;
/// use sdivi_parsing::feature_record::FeatureRecord;
///
/// let records: Vec<FeatureRecord> = vec![];
/// let dg = build_dependency_graph_with_tsconfig(&records, None, None);
/// assert_eq!(dg.node_count(), 0);
/// ```
#[cfg(feature = "pipeline-records")]
pub fn build_dependency_graph_with_tsconfig(
    records: &[sdivi_parsing::feature_record::FeatureRecord],
    go_module: Option<&str>,
    tsconfig: Option<&crate::tsconfig::TsConfigPaths>,
) -> DependencyGraph {
    use crate::resolve::{build_stem_map, compute_java_roots, resolve_imports};

    let mut graph: Graph<PathBuf, ()> = Graph::new();
    let mut path_to_node: BTreeMap<PathBuf, NodeIndex> = BTreeMap::new();

    for record in records {
        let ni = graph.add_node(record.path.clone());
        path_to_node.insert(record.path.clone(), ni);
    }

    let stem_map = build_stem_map(&path_to_node);
    let java_roots = compute_java_roots(&path_to_node);

    for record in records {
        let from_ni = path_to_node[&record.path];
        for import in &record.imports {
            let targets = resolve_imports(
                import,
                &record.path,
                &record.language,
                &stem_map,
                &path_to_node,
                go_module,
                &java_roots,
                tsconfig,
            );
            if targets.is_empty() {
                debug!(%import, path = ?record.path, "unresolved import dropped");
            }
            for to_ni in targets {
                if to_ni != from_ni && !graph.contains_edge(from_ni, to_ni) {
                    graph.add_edge(from_ni, to_ni, ());
                }
            }
        }
    }

    DependencyGraph {
        graph,
        path_to_node,
    }
}
