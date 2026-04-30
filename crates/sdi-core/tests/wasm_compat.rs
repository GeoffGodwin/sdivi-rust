//! WASM-compatibility smoke tests.
//!
//! These tests run on all platforms to verify that the pure-compute API
//! compiles and produces sensible outputs without any I/O.
//!
//! The actual `wasm32-unknown-unknown` build is verified in CI via:
//!   cargo build -p sdi-core --target wasm32-unknown-unknown --no-default-features

use sdi_core::compute::boundaries::detect_boundaries;
use sdi_core::compute::coupling::compute_coupling_topology;
use sdi_core::compute::patterns::compute_pattern_metrics;
use sdi_core::compute::thresholds::compute_thresholds_check;
use sdi_core::input::{DependencyGraphInput, LeidenConfigInput, ThresholdsInput};
use sdi_core::normalize_and_hash;
use sdi_snapshot::delta::null_summary;

#[test]
fn coupling_topology_callable() {
    let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
    let r = compute_coupling_topology(&g).unwrap();
    assert_eq!(r.node_count, 0);
}

#[test]
fn detect_boundaries_callable() {
    let g = DependencyGraphInput { nodes: vec![], edges: vec![] };
    let r = detect_boundaries(&g, &LeidenConfigInput::default(), &[]).unwrap();
    assert_eq!(r.community_count, 0);
    assert_eq!(r.historical_stability, 0.0);
}

#[test]
fn pattern_metrics_callable() {
    let r = compute_pattern_metrics(&[]);
    assert_eq!(r.total_entropy, 0.0);
}

#[test]
fn thresholds_check_callable() {
    let r = compute_thresholds_check(&null_summary(), &ThresholdsInput::default());
    assert!(!r.breached);
}

#[test]
fn normalize_and_hash_callable() {
    let h = normalize_and_hash("try_expression", &[]);
    assert_eq!(h.len(), 64);
}
