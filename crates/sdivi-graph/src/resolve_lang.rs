//! Language-specific import resolvers for Python, Go, and Java.
//!
//! Called from [`crate::resolve::resolve_imports`] after language dispatch.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use petgraph::graph::NodeIndex;
use tracing::debug;

// ─── Python ──────────────────────────────────────────────────────────────────

pub(crate) fn resolve_python_bare(
    specifier: &str,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    let rel = specifier.replace('.', "/");
    if let Some(&ni) = path_to_node.get(&PathBuf::from(format!("{rel}.py"))) {
        return vec![ni];
    }
    if let Some(&ni) = path_to_node.get(&PathBuf::from(&rel).join("__init__.py")) {
        return vec![ni];
    }
    // PEP 420 namespace package: directory containing any .py file (no __init__.py).
    let dir = PathBuf::from(&rel);
    for (p, &ni) in path_to_node {
        if p.starts_with(&dir) && p.extension().map(|e| e == "py").unwrap_or(false) {
            return vec![ni];
        }
    }
    vec![]
}

pub(crate) fn resolve_python_relative(
    specifier: &str,
    from_path: &Path,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    let dot_count = specifier.chars().take_while(|&c| c == '.').count();
    let remainder = &specifier[dot_count..];
    let levels_up = dot_count.saturating_sub(1);

    let from_dir = match from_path.parent() {
        Some(d) => d.to_path_buf(),
        None => return vec![],
    };
    let mut base = from_dir;
    for _ in 0..levels_up {
        match base.parent().map(|p| p.to_path_buf()) {
            Some(p) => base = p,
            None => {
                debug!(%specifier, path = ?from_path, "Python relative import overshoots root");
                return vec![];
            }
        }
    }

    if remainder.is_empty() {
        return path_to_node
            .get(&base.join("__init__.py"))
            .copied()
            .into_iter()
            .collect();
    }
    let rel = remainder.replace('.', "/");
    if let Some(&ni) = path_to_node.get(&base.join(format!("{rel}.py"))) {
        return vec![ni];
    }
    if let Some(&ni) = path_to_node.get(&base.join(&rel).join("__init__.py")) {
        return vec![ni];
    }
    vec![]
}

// ─── Go ──────────────────────────────────────────────────────────────────────

pub(crate) fn resolve_go_module(
    specifier: &str,
    go_mod_prefix: &str,
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    let Some(rest) = specifier.strip_prefix(go_mod_prefix) else {
        debug!(%specifier, "go import not in module prefix, treated as external");
        return vec![];
    };
    let rel = rest.trim_start_matches('/');
    if rel.is_empty() {
        return vec![];
    }
    let dir = PathBuf::from(rel);
    path_to_node
        .iter()
        .filter(|(p, _)| {
            p.parent() == Some(dir.as_path()) && p.extension().map(|e| e == "go").unwrap_or(false)
        })
        .map(|(_, &ni)| ni)
        .collect()
}

// ─── Java ────────────────────────────────────────────────────────────────────

pub(crate) fn resolve_java_dotted(
    specifier: &str,
    java_roots: &[PathBuf],
    path_to_node: &BTreeMap<PathBuf, NodeIndex>,
) -> Vec<NodeIndex> {
    if let Some(pkg) = specifier.strip_suffix(".*") {
        // Wildcard: one edge per .java file directly inside the package directory.
        let dir_rel = pkg.replace('.', "/");
        let mut results = vec![];
        for root in java_roots {
            let dir = if root.as_os_str().is_empty() {
                PathBuf::from(&dir_rel)
            } else {
                root.join(&dir_rel)
            };
            for (p, &ni) in path_to_node {
                if p.parent() == Some(dir.as_path())
                    && p.extension().map(|e| e == "java").unwrap_or(false)
                {
                    results.push(ni);
                }
            }
        }
        results
    } else {
        let class_rel = specifier.replace('.', "/") + ".java";
        for root in java_roots {
            let candidate = if root.as_os_str().is_empty() {
                PathBuf::from(&class_rel)
            } else {
                root.join(&class_rel)
            };
            if let Some(&ni) = path_to_node.get(&candidate) {
                return vec![ni];
            }
        }
        vec![]
    }
}
