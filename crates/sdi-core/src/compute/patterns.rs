//! [`compute_pattern_metrics`] — entropy and convention drift from input instances.

use std::collections::BTreeMap;

use crate::input::PatternInstanceInput;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::snapshot::PatternMetricsResult;

/// Computes pattern metrics from a slice of pattern instances.
///
/// ## Metrics
///
/// - `entropy_per_category`: Shannon entropy of fingerprint distribution within
///   each category.
/// - `total_entropy`: sum of per-category entropies.
/// - `convention_drift`: for each category, `distinct_fingerprints / total_instances`,
///   averaged across all categories.  In `[0, 1]`; `0.0` when no instances.
/// - `convention_drift_per_category`: per-category drift before averaging.
///
/// # Examples
///
/// ```rust
/// use sdi_core::compute::patterns::compute_pattern_metrics;
/// use sdi_core::input::PatternInstanceInput;
///
/// let patterns = &[PatternInstanceInput {
///     fingerprint: "a".repeat(64),
///     category: "error_handling".to_string(),
///     node_id: "src/lib.rs".to_string(),
///     location: None,
/// }];
/// let result = compute_pattern_metrics(patterns);
/// assert_eq!(result.total_entropy, 0.0);
/// assert!(result.convention_drift_per_category.contains_key("error_handling"));
/// ```
pub fn compute_pattern_metrics(patterns: &[PatternInstanceInput]) -> PatternMetricsResult {
    if patterns.is_empty() {
        return PatternMetricsResult::default();
    }

    // Group by category: category → (fingerprint → count)
    let mut by_category: BTreeMap<&str, BTreeMap<&str, u64>> = BTreeMap::new();
    for p in patterns {
        *by_category
            .entry(p.category.as_str())
            .or_default()
            .entry(p.fingerprint.as_str())
            .or_insert(0) += 1;
    }

    let mut entropy_per_category: BTreeMap<String, f64> = BTreeMap::new();
    let mut convention_drift_per_category: BTreeMap<String, f64> = BTreeMap::new();

    for (category, fingerprints) in &by_category {
        let total: u64 = fingerprints.values().sum();
        let total_f = total as f64;

        // Shannon entropy: -sum(p * log2(p))
        let entropy: f64 = fingerprints
            .values()
            .map(|&c| {
                let p = c as f64 / total_f;
                if p > 0.0 { -p * p.log2() } else { 0.0 }
            })
            .sum();

        entropy_per_category.insert(category.to_string(), entropy);

        let distinct = fingerprints.len() as f64;
        convention_drift_per_category.insert(category.to_string(), distinct / total_f);
    }

    let n_categories = by_category.len() as f64;
    let total_entropy: f64 = entropy_per_category.values().sum();
    let convention_drift = convention_drift_per_category.values().sum::<f64>() / n_categories;

    PatternMetricsResult {
        entropy_per_category,
        total_entropy,
        convention_drift,
        convention_drift_per_category,
    }
}

/// Computes pattern metrics from a pre-assembled [`PatternCatalog`].
///
/// Uses `sdi_patterns::compute_entropy` (normalized Shannon entropy) to keep
/// entropy logic in one place across the pipeline and pure-compute paths.
///
/// # Examples
///
/// ```rust
/// use sdi_core::compute::patterns::compute_pattern_metrics_from_catalog;
/// use sdi_core::PatternCatalog;
///
/// let catalog = PatternCatalog::default();
/// let result = compute_pattern_metrics_from_catalog(&catalog);
/// assert_eq!(result.total_entropy, 0.0);
/// ```
pub fn compute_pattern_metrics_from_catalog(catalog: &PatternCatalog) -> PatternMetricsResult {
    use sdi_patterns::compute_entropy;

    let entropy_per_category: BTreeMap<String, f64> = catalog
        .entries
        .iter()
        .map(|(cat, stats)| (cat.clone(), compute_entropy(stats)))
        .collect();

    let total_entropy: f64 = entropy_per_category.values().sum();

    let convention_drift_per_category: BTreeMap<String, f64> = catalog
        .entries
        .iter()
        .map(|(cat, stats)| {
            let distinct = stats.len() as f64;
            let total: f64 = stats.values().map(|s| f64::from(s.count)).sum();
            (cat.clone(), if total > 0.0 { distinct / total } else { 0.0 })
        })
        .collect();

    let convention_drift = if convention_drift_per_category.is_empty() {
        0.0
    } else {
        convention_drift_per_category.values().sum::<f64>()
            / convention_drift_per_category.len() as f64
    };

    PatternMetricsResult {
        entropy_per_category,
        total_entropy,
        convention_drift,
        convention_drift_per_category,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::PatternInstanceInput;

    fn inst(category: &str, fp: &str) -> PatternInstanceInput {
        PatternInstanceInput {
            fingerprint: fp.to_string(),
            category: category.to_string(),
            node_id: "src/lib.rs".to_string(),
            location: None,
        }
    }

    #[test]
    fn empty_input_returns_defaults() {
        let r = compute_pattern_metrics(&[]);
        assert_eq!(r.total_entropy, 0.0);
        assert_eq!(r.convention_drift, 0.0);
        assert!(r.entropy_per_category.is_empty());
    }

    #[test]
    fn single_fingerprint_entropy_is_zero() {
        let patterns = vec![inst("error_handling", "aaa"), inst("error_handling", "aaa")];
        let r = compute_pattern_metrics(&patterns);
        // One fingerprint → p=1.0 → entropy = 0
        assert!((r.entropy_per_category["error_handling"]).abs() < 1e-10);
    }

    #[test]
    fn two_equal_fingerprints_entropy_is_one() {
        let patterns = vec![inst("error_handling", "aaa"), inst("error_handling", "bbb")];
        let r = compute_pattern_metrics(&patterns);
        // 50/50 → H = 1.0 bit
        assert!((r.entropy_per_category["error_handling"] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn convention_drift_one_distinct_per_instance() {
        let patterns = vec![inst("cat_a", "fp1"), inst("cat_a", "fp2")];
        let r = compute_pattern_metrics(&patterns);
        // 2 distinct / 2 total = 1.0
        assert!((r.convention_drift - 1.0).abs() < 1e-10);
    }

    #[test]
    fn convention_drift_all_same() {
        let patterns = vec![inst("cat_a", "fp1"), inst("cat_a", "fp1")];
        let r = compute_pattern_metrics(&patterns);
        // 1 distinct / 2 total = 0.5
        assert!((r.convention_drift - 0.5).abs() < 1e-10);
    }
}
