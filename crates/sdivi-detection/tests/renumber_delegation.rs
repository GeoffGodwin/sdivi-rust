//! Tests for the renumber delegation fix in `refine_partition`.
//!
//! This test ensures that the delegation of `renumber_in_place` to `super::renumber`
//! in `refine.rs` correctly produces densely renumbered community IDs starting from 0.

use rand::rngs::StdRng;
use rand::SeedableRng as _;
use sdivi_detection::internal::{refine_partition, LeidenGraph};
use sdivi_detection::partition::QualityFunction;

fn make_rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

/// Verifies that `refine_partition` produces densely renumbered IDs starting from 0
/// with no gaps or out-of-order IDs.
#[test]
fn refine_partition_produces_dense_renumbering_from_zero() {
    // Create a simple graph with 6 nodes and some structure.
    let g = LeidenGraph::from_edges(6, &[(0, 1), (1, 2), (0, 2), (3, 4), (4, 5), (3, 5)]);
    // Start with all nodes in the same coarse community.
    let coarse = vec![0usize; 6];

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    // Collect all unique community IDs.
    let mut community_ids: Vec<usize> = refined.iter().copied().collect();
    community_ids.sort_unstable();
    community_ids.dedup();

    // Verify the community IDs form a dense range [0, k).
    let k = community_ids.len();
    assert!(
        k > 0,
        "refined partition should have at least one community"
    );
    assert_eq!(
        community_ids,
        (0..k).collect::<Vec<_>>(),
        "community IDs should be dense [0, {}), without gaps or duplicates",
        k
    );
}

/// Verifies density for a more complex graph with multiple expected sub-communities.
#[test]
fn refine_partition_dense_numbering_on_ring_of_cliques() {
    // Ring of three cliques: (0,1,2), (3,4,5), (6,7,8) with cross-bridges.
    let edges = vec![
        // Clique 1.
        (0, 1),
        (1, 2),
        (0, 2),
        // Clique 2.
        (3, 4),
        (4, 5),
        (3, 5),
        // Clique 3.
        (6, 7),
        (7, 8),
        (6, 8),
        // Cross-bridges.
        (2, 3),
        (5, 6),
        (8, 0),
    ];
    let g = LeidenGraph::from_edges(9, &edges);
    let coarse = vec![0usize; 9];

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    // Extract and check density.
    let mut ids: Vec<usize> = refined.iter().copied().collect();
    ids.sort_unstable();
    ids.dedup();

    let k = ids.len();
    assert_eq!(
        ids,
        (0..k).collect::<Vec<_>>(),
        "refined partition must have dense IDs [0, {}), no gaps",
        k
    );
}

/// Verifies that renumbering is deterministic and stable with the same seed.
#[test]
fn refine_partition_renumbering_deterministic_with_seed() {
    let g = LeidenGraph::from_edges(4, &[(0, 1), (1, 2), (2, 3)]);
    let coarse = vec![0usize; 4];

    let refined1 = refine_partition(
        &g,
        &coarse,
        &mut make_rng(100),
        &QualityFunction::Modularity,
        1.0,
    );

    let refined2 = refine_partition(
        &g,
        &coarse,
        &mut make_rng(100),
        &QualityFunction::Modularity,
        1.0,
    );

    // The renumbered IDs should be identical (deterministic).
    assert_eq!(
        refined1, refined2,
        "same seed should produce identical refinement"
    );
}

/// Verifies that all community IDs are within the range [0, max) where max is the
/// max ID + 1, ensuring no out-of-order or negative IDs.
#[test]
fn refine_partition_all_ids_in_valid_range() {
    let g = LeidenGraph::from_edges(
        8,
        &[
            (0, 1),
            (1, 2),
            (2, 3),
            (0, 3),
            (4, 5),
            (5, 6),
            (6, 7),
            (4, 7),
            (3, 4),
        ],
    );
    let coarse = vec![0usize; 8];

    let refined = refine_partition(
        &g,
        &coarse,
        &mut make_rng(42),
        &QualityFunction::Modularity,
        1.0,
    );

    let max_id = refined.iter().copied().max().unwrap_or(0);
    let min_id = refined.iter().copied().min().unwrap_or(0);

    assert_eq!(min_id, 0, "minimum community ID should be 0");
    assert!(
        max_id < refined.len() as usize,
        "maximum ID must be less than node count"
    );

    // Check that all IDs in [0, max_id] are present (density).
    let mut present = vec![false; max_id + 1];
    for &id in &refined {
        present[id] = true;
    }

    for (i, &was_present) in present.iter().enumerate() {
        assert!(
            was_present,
            "community ID {} is missing; IDs must be contiguous from 0",
            i
        );
    }
}
