use std::collections::BTreeMap;

use sdivi_snapshot::boundary_inference::{infer_boundaries, PriorPartition};

fn partition_from(pairs: &[(&str, u32)]) -> PriorPartition {
    PriorPartition {
        cluster_assignments: pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
    }
}

#[test]
fn empty_input_no_proposals() {
    let r = infer_boundaries(&[], 3);
    assert!(r.proposals.is_empty());
    assert_eq!(r.partition_count, 0);
}

#[test]
fn single_partition_no_proposals() {
    // Only 1 partition — no pairs to evaluate.
    let p = partition_from(&[("a.rs", 0), ("b.rs", 0)]);
    let r = infer_boundaries(&[p], 1);
    assert!(r.proposals.is_empty());
}

#[test]
fn threshold_met_community_proposed() {
    // 3 identical partitions → 2 consecutive pairs → threshold=2 → community proposed.
    let make = || partition_from(&[("a.rs", 0), ("b.rs", 0), ("c.rs", 1)]);
    let r = infer_boundaries(&[make(), make(), make()], 2);
    assert!(!r.proposals.is_empty());
    // Community {a.rs, b.rs} should be in proposals.
    let found = r
        .proposals
        .iter()
        .any(|p| p.node_ids.contains(&"a.rs".to_string()));
    assert!(found);
}

#[test]
fn threshold_not_met_no_proposal() {
    // 2 partitions → 1 pair → threshold=2 → not enough.
    let make = || partition_from(&[("a.rs", 0), ("b.rs", 0)]);
    let r = infer_boundaries(&[make(), make()], 2);
    assert!(r.proposals.is_empty());
}

#[test]
fn community_id_renaming_ignored() {
    // Same node sets but different numeric IDs across snapshots → should still
    // be treated as stable.
    let p1: PriorPartition = PriorPartition {
        cluster_assignments: BTreeMap::from([
            ("a.rs".to_string(), 0u32),
            ("b.rs".to_string(), 0u32),
        ]),
    };
    let p2: PriorPartition = PriorPartition {
        cluster_assignments: BTreeMap::from([
            ("a.rs".to_string(), 99u32), // numeric ID changed
            ("b.rs".to_string(), 99u32),
        ]),
    };
    let p3: PriorPartition = PriorPartition {
        cluster_assignments: BTreeMap::from([
            ("a.rs".to_string(), 5u32),
            ("b.rs".to_string(), 5u32),
        ]),
    };
    let r = infer_boundaries(&[p1, p2, p3], 2);
    // Node set {a.rs, b.rs} stable for 2 pairs → proposed.
    assert!(!r.proposals.is_empty());
}

#[test]
fn partition_count_matches_input_length() {
    let partitions: Vec<PriorPartition> = (0..5).map(|_| partition_from(&[("x.rs", 0)])).collect();
    let r = infer_boundaries(&partitions, 10);
    assert_eq!(r.partition_count, 5);
}

#[test]
fn proposals_sorted_by_community_id() {
    let make = || partition_from(&[("a.rs", 2), ("b.rs", 2), ("c.rs", 0), ("d.rs", 0)]);
    let r = infer_boundaries(&[make(), make(), make()], 2);
    let ids: Vec<u32> = r.proposals.iter().map(|p| p.community_id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(ids, sorted);
}
