use sdi_core::compute::patterns::compute_pattern_metrics;
use sdi_core::input::PatternInstanceInput;

fn inst(category: &str, fp: &str) -> PatternInstanceInput {
    PatternInstanceInput {
        fingerprint: fp.to_string(),
        category: category.to_string(),
        node_id: "src/lib.rs".to_string(),
        location: None,
    }
}

#[test]
fn empty_slice_returns_defaults() {
    let r = compute_pattern_metrics(&[]);
    assert_eq!(r.total_entropy, 0.0);
    assert_eq!(r.convention_drift, 0.0);
    assert!(r.entropy_per_category.is_empty());
}

#[test]
fn single_fingerprint_entropy_is_zero() {
    let patterns = vec![inst("error_handling", "aaa"), inst("error_handling", "aaa")];
    let r = compute_pattern_metrics(&patterns);
    assert!((r.entropy_per_category["error_handling"]).abs() < 1e-10);
}

#[test]
fn two_equal_fingerprints_entropy_is_one() {
    let patterns = vec![inst("cat", "fp1"), inst("cat", "fp2")];
    let r = compute_pattern_metrics(&patterns);
    assert!((r.entropy_per_category["cat"] - 1.0).abs() < 1e-10);
}

#[test]
fn convention_drift_all_distinct() {
    let patterns = vec![inst("cat", "fp1"), inst("cat", "fp2")];
    let r = compute_pattern_metrics(&patterns);
    // 2 distinct / 2 total = 1.0
    assert!((r.convention_drift - 1.0).abs() < 1e-10);
}

#[test]
fn convention_drift_all_same() {
    let patterns = vec![inst("cat", "fp1"), inst("cat", "fp1")];
    let r = compute_pattern_metrics(&patterns);
    // 1 distinct / 2 total = 0.5
    assert!((r.convention_drift - 0.5).abs() < 1e-10);
}

#[test]
fn total_entropy_sums_per_category() {
    let patterns = vec![
        inst("cat_a", "f1"), inst("cat_a", "f2"),
        inst("cat_b", "g1"), inst("cat_b", "g2"),
    ];
    let r = compute_pattern_metrics(&patterns);
    let expected = r.entropy_per_category.values().sum::<f64>();
    assert!((r.total_entropy - expected).abs() < 1e-10);
}

#[test]
fn convention_drift_averaged_across_categories() {
    // cat_a: 2/2 = 1.0  cat_b: 1/2 = 0.5  avg = 0.75
    let patterns = vec![
        inst("cat_a", "f1"), inst("cat_a", "f2"),
        inst("cat_b", "g1"), inst("cat_b", "g1"),
    ];
    let r = compute_pattern_metrics(&patterns);
    assert!((r.convention_drift - 0.75).abs() < 1e-10);
}

#[test]
fn convention_drift_in_zero_one_range() {
    let patterns = vec![
        inst("x", "a"), inst("x", "b"), inst("x", "c"),
        inst("y", "d"),
    ];
    let r = compute_pattern_metrics(&patterns);
    assert!(r.convention_drift >= 0.0 && r.convention_drift <= 1.0);
}
