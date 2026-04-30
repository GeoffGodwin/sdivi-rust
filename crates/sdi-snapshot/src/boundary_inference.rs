//! [`infer_boundaries`] — propose community boundaries from partition history.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A proposed boundary derived from consecutive-snapshot community stability.
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::boundary_inference::BoundaryProposal;
///
/// let proposal = BoundaryProposal {
///     community_id: 0,
///     stable_snapshots: 3,
///     node_ids: vec!["src/lib.rs".to_string()],
/// };
/// assert_eq!(proposal.community_id, 0);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryProposal {
    /// The community ID (from the most recent partition).
    pub community_id: u32,
    /// Number of consecutive snapshots this community has been stable.
    pub stable_snapshots: u32,
    /// Node IDs (paths) belonging to this community in the most recent partition.
    pub node_ids: Vec<String>,
}

/// Result of [`infer_boundaries`].
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::boundary_inference::infer_boundaries;
///
/// let result = infer_boundaries(&[], 3);
/// assert!(result.proposals.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BoundaryInferenceResult {
    /// Communities that have been stable for at least `stability_threshold`
    /// consecutive snapshots, proposed as boundary candidates.
    pub proposals: Vec<BoundaryProposal>,
    /// Number of partitions considered (the length of the input slice).
    pub partition_count: usize,
}

/// A prior partition supplied to boundary inference.
///
/// Each entry maps a node ID (repo-relative path string) to its community ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriorPartition {
    /// Node ID → community ID mapping.
    pub cluster_assignments: BTreeMap<String, u32>,
}

/// Infers boundary proposals from a sequence of prior partitions.
///
/// `partitions` must be ordered oldest → newest.  The most recent entry is the
/// proposal source; the earlier entries supply the stability history.
///
/// A community is proposed as a boundary candidate when it has been present
/// (same set of node IDs, regardless of numeric ID churn) in at least
/// `stability_threshold` consecutive partition pairs ending at the most recent
/// partition.
///
/// **Stability definition:** two consecutive partitions agree on a community
/// when the same set of node IDs is assigned to the same logical group.
/// Community IDs may renumber across snapshots; we match by node-set membership.
///
/// Returns an empty `proposals` list when:
/// - `partitions` is empty
/// - the slice has fewer entries than `stability_threshold`
///
/// # Examples
///
/// ```rust
/// use sdi_snapshot::boundary_inference::{infer_boundaries, PriorPartition};
/// use std::collections::BTreeMap;
///
/// let result = infer_boundaries(&[], 3);
/// assert!(result.proposals.is_empty());
/// assert_eq!(result.partition_count, 0);
/// ```
pub fn infer_boundaries(
    partitions: &[PriorPartition],
    stability_threshold: u32,
) -> BoundaryInferenceResult {
    let partition_count = partitions.len();

    if partition_count == 0 || stability_threshold == 0 {
        return BoundaryInferenceResult {
            proposals: vec![],
            partition_count,
        };
    }

    let latest = partitions.last().unwrap();
    let latest_communities = invert_assignments(&latest.cluster_assignments);

    // For each community in the latest partition, count how many consecutive
    // pairs (ending at latest) agree on that node set.
    let mut proposals: Vec<BoundaryProposal> = Vec::new();

    for (comm_id, node_set) in &latest_communities {
        let stable = count_stable_tail(partitions, node_set);
        if stable >= stability_threshold {
            let mut node_ids: Vec<String> = node_set.iter().cloned().collect();
            node_ids.sort();
            proposals.push(BoundaryProposal {
                community_id: *comm_id,
                stable_snapshots: stable,
                node_ids,
            });
        }
    }

    proposals.sort_by_key(|p| p.community_id);

    BoundaryInferenceResult {
        proposals,
        partition_count,
    }
}

/// Inverts a node→community map to community→{node_ids} map.
fn invert_assignments(
    assignments: &BTreeMap<String, u32>,
) -> BTreeMap<u32, std::collections::BTreeSet<String>> {
    let mut result: BTreeMap<u32, std::collections::BTreeSet<String>> = BTreeMap::new();
    for (node, &comm) in assignments {
        result.entry(comm).or_default().insert(node.clone());
    }
    result
}

/// Counts how many consecutive pairs ending at the last partition agree on
/// `node_set` (i.e., that exact node set forms a community in each partition).
fn count_stable_tail(
    partitions: &[PriorPartition],
    node_set: &std::collections::BTreeSet<String>,
) -> u32 {
    if partitions.len() < 2 {
        // Only one partition; no pairs to check.
        return 0;
    }

    let mut stable = 0u32;
    // Walk pairs from newest-1 backwards.
    for i in (0..partitions.len() - 1).rev() {
        let p = &partitions[i];
        let communities = invert_assignments(&p.cluster_assignments);
        let found = communities.values().any(|s| s == node_set);
        if found {
            stable += 1;
        } else {
            break;
        }
    }
    stable
}

#[cfg(test)]
mod tests {
    use super::*;

    fn partition_from(pairs: &[(&str, u32)]) -> PriorPartition {
        PriorPartition {
            cluster_assignments: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect(),
        }
    }

    #[test]
    fn empty_input_returns_no_proposals() {
        let r = infer_boundaries(&[], 3);
        assert!(r.proposals.is_empty());
        assert_eq!(r.partition_count, 0);
    }

    #[test]
    fn single_partition_no_proposals_due_to_no_history() {
        let p = partition_from(&[("a", 0), ("b", 0), ("c", 1)]);
        let r = infer_boundaries(&[p], 2);
        // threshold 2 but only 1 partition → no pairs to check → 0 stable → no proposals
        assert!(r.proposals.is_empty());
    }

    #[test]
    fn stable_community_proposed() {
        // Three partitions; community {a, b} stable across all three pairs.
        let p1 = partition_from(&[("a", 0), ("b", 0), ("c", 1)]);
        let p2 = partition_from(&[("a", 0), ("b", 0), ("c", 1)]);
        let p3 = partition_from(&[("a", 0), ("b", 0), ("c", 1)]);
        let r = infer_boundaries(&[p1, p2, p3], 2);
        // community 0 ({a,b}) stable for 2 pairs → should be proposed
        assert!(!r.proposals.is_empty());
        let prop = r.proposals.iter().find(|p| p.node_ids == vec!["a", "b"]);
        assert!(prop.is_some());
    }

    #[test]
    fn threshold_not_met_no_proposal() {
        let p1 = partition_from(&[("a", 0), ("b", 0)]);
        let p2 = partition_from(&[("a", 0), ("b", 0)]);
        let r = infer_boundaries(&[p1, p2], 3);
        // only 1 pair, threshold=3 → no proposals
        assert!(r.proposals.is_empty());
    }
}
