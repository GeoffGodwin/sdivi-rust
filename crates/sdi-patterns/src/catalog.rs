//! [`PatternCatalog`] — per-category pattern fingerprint aggregation.

use std::collections::BTreeMap;
use std::path::PathBuf;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use sdi_config::PatternsConfig;
use sdi_parsing::feature_record::FeatureRecord;

use crate::fingerprint::{fingerprint_node_kind, PatternFingerprint};
use crate::queries;

/// The file path and source position of a single pattern instance.
///
/// # Examples
///
/// ```rust
/// use std::path::PathBuf;
/// use sdi_patterns::catalog::PatternLocation;
///
/// let loc = PatternLocation { file: PathBuf::from("src/lib.rs"), start_row: 10, start_col: 4 };
/// assert_eq!(loc.start_row, 10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternLocation {
    /// Source file path relative to the repository root.
    pub file: PathBuf,
    /// Zero-indexed source row (line) of the pattern instance.
    pub start_row: usize,
    /// Zero-indexed source column of the pattern instance.
    pub start_col: usize,
}

/// Aggregated statistics for a single pattern fingerprint within one category.
///
/// # Examples
///
/// ```rust
/// use sdi_patterns::catalog::PatternStats;
///
/// let stats = PatternStats { count: 3, locations: vec![] };
/// assert_eq!(stats.count, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStats {
    /// Total number of instances across all non-excluded files.
    pub count: u32,
    /// All source locations where this fingerprint was observed.
    pub locations: Vec<PatternLocation>,
}

/// Per-category pattern catalog keyed by [`PatternFingerprint`].
///
/// `BTreeMap` ordering guarantees deterministic serialization.
/// Empty categories are omitted from the map.
///
/// # Examples
///
/// ```rust
/// use sdi_config::Config;
/// use sdi_patterns::catalog::build_catalog;
///
/// let catalog = build_catalog(&[], &Config::default().patterns);
/// assert!(catalog.entries.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PatternCatalog {
    /// Outer key: category name; inner key: pattern fingerprint.
    pub entries: BTreeMap<String, BTreeMap<PatternFingerprint, PatternStats>>,
}

/// Builds a [`PatternCatalog`] from parsed source records and the patterns config.
///
/// Files whose path matches any glob in `config.scope_exclude` are skipped for
/// pattern collection but remain in the graph and partition stages. Fingerprints
/// with a total instance count below `config.min_pattern_nodes` are removed.
///
/// `config.categories = "auto"` activates all five built-in categories.
///
/// # Examples
///
/// ```rust
/// use sdi_config::Config;
/// use sdi_patterns::catalog::build_catalog;
///
/// let catalog = build_catalog(&[], &Config::default().patterns);
/// assert!(catalog.entries.is_empty());
/// ```
pub fn build_catalog(records: &[FeatureRecord], config: &PatternsConfig) -> PatternCatalog {
    let exclude_set = build_globset(&config.scope_exclude);

    let mut entries: BTreeMap<String, BTreeMap<PatternFingerprint, PatternStats>> =
        BTreeMap::new();

    for record in records {
        if is_excluded(&record.path, &exclude_set) {
            continue;
        }
        for hint in &record.pattern_hints {
            let Some(category) =
                queries::category_for_node_kind(&hint.node_kind, &record.language)
            else {
                continue;
            };
            let fp = fingerprint_node_kind(&hint.node_kind);
            let location = PatternLocation {
                file: record.path.clone(),
                start_row: hint.start_row,
                start_col: hint.start_col,
            };
            let cat_map = entries.entry(category.to_string()).or_default();
            let stats = cat_map.entry(fp).or_insert(PatternStats {
                count: 0,
                locations: vec![],
            });
            stats.count += 1;
            stats.locations.push(location);
        }
    }

    let min = config.min_pattern_nodes;
    for cat_map in entries.values_mut() {
        cat_map.retain(|_, stats| stats.count >= min);
    }
    entries.retain(|_, cat_map| !cat_map.is_empty());

    PatternCatalog { entries }
}

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        if let Ok(glob) = Glob::new(pat) {
            builder.add(glob);
        }
    }
    builder.build().ok()
}

fn is_excluded(path: &std::path::Path, exclude_set: &Option<GlobSet>) -> bool {
    match exclude_set {
        None => false,
        Some(gs) => gs.is_match(path),
    }
}

#[cfg(test)]
mod tests {
    use sdi_config::PatternsConfig;
    use sdi_parsing::feature_record::{FeatureRecord, PatternHint};

    use super::*;

    fn make_record(path: &str, hints: Vec<PatternHint>) -> FeatureRecord {
        FeatureRecord {
            path: PathBuf::from(path),
            language: "rust".to_string(),
            imports: vec![],
            exports: vec![],
            signatures: vec![],
            pattern_hints: hints,
        }
    }

    fn make_hint(node_kind: &str) -> PatternHint {
        PatternHint {
            node_kind: node_kind.to_string(),
            start_byte: 0,
            end_byte: 10,
            start_row: 0,
            start_col: 0,
            text: "stub".to_string(),
        }
    }

    #[test]
    fn empty_records_produces_empty_catalog() {
        let config = PatternsConfig::default();
        let catalog = build_catalog(&[], &config);
        assert!(catalog.entries.is_empty());
    }

    #[test]
    fn try_expression_appears_in_error_handling() {
        let hints = vec![make_hint("try_expression")];
        let records = vec![make_record("src/lib.rs", hints)];
        let mut config = PatternsConfig::default();
        config.min_pattern_nodes = 1;
        let catalog = build_catalog(&records, &config);
        assert!(catalog.entries.contains_key("error_handling"));
    }

    #[test]
    fn scope_exclude_removes_file_from_catalog() {
        let hints = vec![make_hint("try_expression")];
        let records = vec![make_record("src/vendor/lib.rs", hints)];
        let mut config = PatternsConfig::default();
        config.min_pattern_nodes = 1;
        config.scope_exclude = vec!["src/vendor/**".to_string()];
        let catalog = build_catalog(&records, &config);
        assert!(catalog.entries.is_empty(), "excluded file must not appear in catalog");
    }

    #[test]
    fn min_pattern_nodes_filters_low_count_fingerprints() {
        let hints = vec![make_hint("try_expression"), make_hint("try_expression")];
        let records = vec![make_record("src/lib.rs", hints)];
        let mut config = PatternsConfig::default();
        config.min_pattern_nodes = 5;
        let catalog = build_catalog(&records, &config);
        assert!(
            catalog.entries.is_empty(),
            "fingerprint with count < min_pattern_nodes must be excluded"
        );
    }
}
