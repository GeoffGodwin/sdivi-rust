//! Historical-stability helpers for [`super::boundaries::detect_boundaries`].

use std::collections::{BTreeMap, BTreeSet};

use crate::input::PriorPartition;

/// Fraction of consecutive prior-partition pairs whose community sets agree.
///
/// Returns `0.0` when `prior` has fewer than two entries (no pairs to compare).
pub(super) fn compute_historical_stability(prior: &[PriorPartition]) -> f64 {
    if prior.len() < 2 {
        return 0.0;
    }

    let n_pairs = (prior.len() - 1) as f64;
    let mut matching = 0.0f64;

    for pair in prior.windows(2) {
        let prev = invert_assignments(&pair[0].cluster_assignments);
        let next = invert_assignments(&pair[1].cluster_assignments);
        if community_sets_match(&prev, &next) {
            matching += 1.0;
        }
    }

    matching / n_pairs
}

fn invert_assignments(
    assignments: &BTreeMap<String, u32>,
) -> BTreeMap<u32, BTreeSet<String>> {
    let mut result: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();
    for (node, &comm) in assignments {
        result.entry(comm).or_default().insert(node.clone());
    }
    result
}

fn community_sets_match(
    a: &BTreeMap<u32, BTreeSet<String>>,
    b: &BTreeMap<u32, BTreeSet<String>>,
) -> bool {
    let a_sets: BTreeSet<_> = a.values().collect();
    let b_sets: BTreeSet<_> = b.values().collect();
    a_sets == b_sets
}
