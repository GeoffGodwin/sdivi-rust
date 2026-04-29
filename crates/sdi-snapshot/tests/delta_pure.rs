use std::collections::BTreeMap;

use proptest::prelude::*;
use sdi_detection::partition::LeidenPartition;
use sdi_graph::metrics::GraphMetrics;
use sdi_patterns::PatternCatalog;
use sdi_snapshot::build_snapshot;
use sdi_snapshot::compute_delta;
use sdi_snapshot::Snapshot;

fn make_snap(density: f64, comm: usize) -> Snapshot {
    let mut stability = BTreeMap::new();
    for i in 0..comm {
        stability.insert(i, 1.0f64);
    }
    build_snapshot(
        GraphMetrics {
            node_count: 2,
            edge_count: 0,
            density,
            cycle_count: 0,
            top_hubs: vec![],
            component_count: 1,
        },
        LeidenPartition {
            assignments: BTreeMap::new(),
            stability,
            modularity: 0.0,
            seed: 42,
        },
        PatternCatalog::default(),
        None,
        "2026-04-29T00:00:00Z",
        None,
    )
}

proptest! {
    /// `compute_delta` is referentially transparent: same inputs always yield the same output.
    #[test]
    fn prop_delta_referentially_transparent(
        density1 in 0.0f64..1.0,
        density2 in 0.0f64..1.0,
        comm1 in 0usize..10,
        comm2 in 0usize..10,
    ) {
        let snap1 = make_snap(density1, comm1);
        let snap2 = make_snap(density2, comm2);
        let d1 = compute_delta(&snap1, &snap2);
        let d2 = compute_delta(&snap1, &snap2);
        prop_assert_eq!(d1, d2);
    }
}
