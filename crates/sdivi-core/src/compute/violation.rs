//! Private helpers for [`super::boundaries::compute_boundary_violations`].

use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::error::AnalysisError;
use crate::input::BoundarySpecInput;

/// A compiled boundary with pre-built glob matcher and metadata.
pub(crate) struct CompiledBoundary {
    pub(crate) name: String,
    /// Parallel to `glob_set`'s pattern order — used for specificity scoring.
    pub(crate) patterns: Vec<String>,
    pub(crate) glob_set: GlobSet,
    pub(crate) allow_imports_from: Vec<String>,
}

/// Compiles every boundary's `modules` globs into [`CompiledBoundary`] entries.
///
/// Glob compilation is O(patterns) and happens once per `compute_boundary_violations`
/// call — not per node.
pub(crate) fn compile_boundaries(
    spec: &BoundarySpecInput,
) -> Result<Vec<CompiledBoundary>, AnalysisError> {
    let mut result = Vec::with_capacity(spec.boundaries.len());
    for def in &spec.boundaries {
        let mut builder = GlobSetBuilder::new();
        for pattern in &def.modules {
            let glob = Glob::new(pattern).map_err(|e| AnalysisError::InvalidConfig {
                message: format!("invalid glob {:?} in boundary {:?}: {e}", pattern, def.name),
            })?;
            builder.add(glob);
        }
        let glob_set = builder.build().map_err(|e| AnalysisError::InvalidConfig {
            message: format!("failed to build GlobSet for boundary {:?}: {e}", def.name),
        })?;
        result.push(CompiledBoundary {
            name: def.name.clone(),
            patterns: def.modules.clone(),
            glob_set,
            allow_imports_from: def.allow_imports_from.clone(),
        });
    }
    Ok(result)
}

/// Returns the name of the **most-specific** boundary matching `node_id`, or `None`.
///
/// # Determinism
///
/// When multiple boundaries match the same node, the boundary whose longest individual
/// glob pattern matched wins. Ties (equal pattern length) are broken by ascending
/// lexicographic order of the boundary name.  This rule is deterministic and
/// independent of map iteration order.
pub(crate) fn match_boundary<'a>(
    node_id: &str,
    compiled: &'a [CompiledBoundary],
) -> Option<&'a str> {
    let mut best_name: Option<&'a str> = None;
    let mut best_len: usize = 0;

    for cb in compiled {
        for idx in cb.glob_set.matches(node_id) {
            let pat_len = cb.patterns[idx].len();
            let take = pat_len > best_len
                || (pat_len == best_len && best_name.is_none_or(|n: &str| cb.name.as_str() < n));
            if take {
                best_name = Some(&cb.name);
                best_len = pat_len;
            }
        }
    }

    best_name
}
