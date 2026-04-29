//! Per-category normalized Shannon entropy for pattern catalogs.

use std::collections::BTreeMap;

use crate::catalog::PatternStats;
use crate::fingerprint::PatternFingerprint;

/// Computes normalized Shannon entropy for the pattern statistics within one category.
///
/// # Formula
///
/// ```text
/// H = -sum(p_i * log2(p_i)) / log2(N)
/// ```
///
/// where:
/// - `p_i = count_i / total_instances` for each distinct fingerprint
/// - `N` = number of distinct fingerprints
/// - Result is bounded to `[0, 1]`
/// - Returns `0.0` when `N <= 1` (only one distinct shape, or no instances)
///
/// # Examples
///
/// ```rust
/// use std::collections::BTreeMap;
/// use sdi_patterns::fingerprint::fingerprint_node_kind;
/// use sdi_patterns::catalog::PatternStats;
/// use sdi_patterns::entropy::compute_entropy;
///
/// let mut stats = BTreeMap::new();
/// let fp1 = fingerprint_node_kind("try_expression");
/// let fp2 = fingerprint_node_kind("match_expression");
/// stats.insert(fp1, PatternStats { count: 5, locations: vec![] });
/// stats.insert(fp2, PatternStats { count: 5, locations: vec![] });
///
/// // Equal distribution across 2 shapes → maximum entropy = 1.0
/// let h = compute_entropy(&stats);
/// assert!((h - 1.0).abs() < 1e-10);
/// ```
pub fn compute_entropy(stats: &BTreeMap<PatternFingerprint, PatternStats>) -> f64 {
    let distinct = stats.len();
    if distinct <= 1 {
        return 0.0;
    }

    let total: u32 = stats.values().map(|s| s.count).sum();
    if total == 0 {
        return 0.0;
    }

    let total_f = f64::from(total);
    let raw: f64 = stats
        .values()
        .map(|s| {
            let p = f64::from(s.count) / total_f;
            if p > 0.0 {
                -p * p.log2()
            } else {
                0.0
            }
        })
        .sum();

    let max = (distinct as f64).log2();
    if max <= 0.0 {
        return 0.0;
    }

    (raw / max).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::fingerprint_node_kind;

    fn make_stats(count: u32) -> PatternStats {
        PatternStats { count, locations: vec![] }
    }

    #[test]
    fn empty_stats_returns_zero() {
        let stats: BTreeMap<PatternFingerprint, PatternStats> = BTreeMap::new();
        assert_eq!(compute_entropy(&stats), 0.0);
    }

    #[test]
    fn single_shape_returns_zero() {
        let mut stats = BTreeMap::new();
        stats.insert(fingerprint_node_kind("try_expression"), make_stats(10));
        assert_eq!(compute_entropy(&stats), 0.0);
    }

    #[test]
    fn equal_distribution_two_shapes_returns_one() {
        let mut stats = BTreeMap::new();
        stats.insert(fingerprint_node_kind("try_expression"), make_stats(5));
        stats.insert(fingerprint_node_kind("match_expression"), make_stats(5));
        let h = compute_entropy(&stats);
        assert!((h - 1.0).abs() < 1e-10);
    }

    #[test]
    fn unequal_distribution_is_between_zero_and_one() {
        let mut stats = BTreeMap::new();
        stats.insert(fingerprint_node_kind("try_expression"), make_stats(9));
        stats.insert(fingerprint_node_kind("match_expression"), make_stats(1));
        let h = compute_entropy(&stats);
        assert!(h > 0.0 && h < 1.0);
    }
}
