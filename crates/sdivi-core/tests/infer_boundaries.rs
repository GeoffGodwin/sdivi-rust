//! Pure-compute tests for `sdivi_core::infer_boundaries`.
//!
//! These tests exercise the same code path used by the consumer app (via WASM)
//! and by the CLI — no filesystem access, caller supplies the prior partitions.

use sdivi_core::{infer_boundaries, SnapshotPriorPartition as PriorPartition};

fn partition(pairs: &[(&str, u32)]) -> PriorPartition {
    PriorPartition {
        cluster_assignments: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    }
}

/// Empty partition history returns no proposals.
#[test]
fn empty_history_returns_no_proposals() {
    let result = infer_boundaries(&[], 3);
    assert!(result.proposals.is_empty());
    assert_eq!(result.partition_count, 0);
}

/// A single partition with threshold > 0 has no stable pairs.
#[test]
fn single_partition_no_stable_pairs() {
    let p = partition(&[("src/lib.rs", 0), ("src/models.rs", 0)]);
    let result = infer_boundaries(&[p], 1);
    assert!(
        result.proposals.is_empty(),
        "need at least 2 partitions to form a stable pair"
    );
}

/// Three identical partitions with threshold 2 proposes all communities.
#[test]
fn three_identical_partitions_threshold_two() {
    let p1 = partition(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1), ("d.rs", 1)]);
    let p2 = partition(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1), ("d.rs", 1)]);
    let p3 = partition(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1), ("d.rs", 1)]);

    let result = infer_boundaries(&[p1, p2, p3], 2);
    assert_eq!(result.partition_count, 3);
    assert_eq!(
        result.proposals.len(),
        2,
        "both communities should be proposed"
    );

    let comm_0 = result
        .proposals
        .iter()
        .find(|p| p.node_ids == vec!["a.rs", "b.rs"]);
    let comm_1 = result
        .proposals
        .iter()
        .find(|p| p.node_ids == vec!["c.rs", "d.rs"]);
    assert!(
        comm_0.is_some(),
        "community with {{a.rs, b.rs}} must be proposed"
    );
    assert!(
        comm_1.is_some(),
        "community with {{c.rs, d.rs}} must be proposed"
    );
}

/// Threshold not met returns no proposals.
#[test]
fn threshold_not_met() {
    let p1 = partition(&[("a.rs", 0), ("b.rs", 0)]);
    let p2 = partition(&[("a.rs", 0), ("b.rs", 0)]);
    let result = infer_boundaries(&[p1, p2], 3);
    assert!(
        result.proposals.is_empty(),
        "only 1 pair available, threshold=3"
    );
}

/// Community renumbering across snapshots (same node sets) is handled correctly.
#[test]
fn community_renumbering_handled() {
    // Community IDs swap between partitions, but node sets are the same.
    let p1 = partition(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1)]);
    let p2 = partition(&[("a.rs", 1), ("b.rs", 1), ("c.rs", 0)]);
    let p3 = partition(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1)]);

    let result = infer_boundaries(&[p1, p2, p3], 2);
    // {a.rs, b.rs} should be stable: pair (p2,p3) has same node set as (p1,p2) after inversion
    // The stability check matches node-sets regardless of community ID labels.
    assert!(
        result
            .proposals
            .iter()
            .any(|p| p.node_ids == vec!["a.rs", "b.rs"]),
        "node-set {{a.rs, b.rs}} should be proposed despite ID churn"
    );
}

/// `zero` threshold produces no proposals (guard against degenerate input).
#[test]
fn zero_threshold_returns_empty() {
    let p = partition(&[("a.rs", 0)]);
    let result = infer_boundaries(&[p.clone(), p], 0);
    assert!(result.proposals.is_empty());
}

/// Proposals are sorted by community_id ascending.
#[test]
fn proposals_sorted_by_community_id() {
    let p1 = partition(&[("a.rs", 5), ("b.rs", 2), ("c.rs", 8)]);
    let p2 = partition(&[("a.rs", 5), ("b.rs", 2), ("c.rs", 8)]);
    let p3 = partition(&[("a.rs", 5), ("b.rs", 2), ("c.rs", 8)]);

    let result = infer_boundaries(&[p1, p2, p3], 2);
    let ids: Vec<u32> = result.proposals.iter().map(|p| p.community_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted, "proposals must be sorted by community_id");
}
