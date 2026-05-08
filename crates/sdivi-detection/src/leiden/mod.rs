//! Native Leiden community detection algorithm.
//!
//! Implements Traag et al. 2019 (arXiv:1810.08473) with Modularity and CPM
//! quality functions.  The algorithm is single-threaded and fully deterministic
//! given the same seed.

pub(crate) mod aggregate;
mod cpm;
pub(crate) mod graph;
mod local_move;
mod modularity;
pub(crate) mod quality;
pub(crate) mod refine;

use rand::rngs::StdRng;
use rand::SeedableRng as _;
use std::collections::BTreeMap;

use sdivi_graph::DependencyGraph;

use crate::partition::{LeidenConfig, LeidenPartition, QualityFunction};
use crate::warm_start::initial_assignment_from_cache;

use aggregate::{aggregate_network, flatten_partition, map_to_aggregate_init, AggregateResult};
use graph::LeidenGraph;
use quality::{compute_modularity, compute_stability};
use refine::refine_partition;

/// Runs the Leiden algorithm on a [`DependencyGraph`].
///
/// Produces a [`LeidenPartition`] with community assignments, per-community
/// stability scores, and overall modularity.
///
/// # Examples
///
/// ```rust
/// use sdivi_detection::leiden::run_leiden;
/// use sdivi_detection::partition::LeidenConfig;
/// use sdivi_graph::dependency_graph::build_dependency_graph_from_edges;
///
/// let dg = build_dependency_graph_from_edges(&[], &[]);
/// let p = run_leiden(&dg, &LeidenConfig::default(), None);
/// assert_eq!(p.community_count(), 0);
/// ```
pub fn run_leiden(
    dg: &DependencyGraph,
    cfg: &LeidenConfig,
    warm_start: Option<&LeidenPartition>,
) -> LeidenPartition {
    let leiden_graph = LeidenGraph::from_dependency_graph(dg);

    if leiden_graph.n == 0 {
        return LeidenPartition {
            assignments: BTreeMap::new(),
            stability: BTreeMap::new(),
            modularity: 0.0,
            seed: cfg.seed,
        };
    }

    let mut rng = StdRng::seed_from_u64(cfg.seed);

    let initial = initial_assignment_from_cache(warm_start, leiden_graph.n);
    let assignment = leiden_recursive(
        &leiden_graph,
        initial,
        &mut rng,
        cfg.max_iterations,
        &cfg.quality,
        cfg.gamma,
        cfg.min_compression_ratio,
        cfg.max_recursion_depth,
        0,
    );

    build_partition(dg, &leiden_graph, assignment, cfg.seed)
}

/// Runs Leiden with per-edge weights supplied via a `(min_idx, max_idx) → weight` map.
///
/// Edges not in `weight_map` get weight `1.0`. Pairs whose endpoints are not
/// connected by an import edge are ignored (KDD-5: change-coupling is a weight
/// modulation, not a new topology).
pub fn run_leiden_with_weights(
    dg: &DependencyGraph,
    cfg: &LeidenConfig,
    warm_start: Option<&LeidenPartition>,
    weight_map: &BTreeMap<(usize, usize), f64>,
) -> LeidenPartition {
    let leiden_graph = LeidenGraph::from_dependency_graph_weighted(dg, weight_map);

    if leiden_graph.n == 0 {
        return LeidenPartition {
            assignments: BTreeMap::new(),
            stability: BTreeMap::new(),
            modularity: 0.0,
            seed: cfg.seed,
        };
    }

    let mut rng = StdRng::seed_from_u64(cfg.seed);
    let initial = initial_assignment_from_cache(warm_start, leiden_graph.n);
    let assignment = leiden_recursive(
        &leiden_graph,
        initial,
        &mut rng,
        cfg.max_iterations,
        &cfg.quality,
        cfg.gamma,
        cfg.min_compression_ratio,
        cfg.max_recursion_depth,
        0,
    );
    build_partition(dg, &leiden_graph, assignment, cfg.seed)
}

/// Recursive Leiden step operating on a graph at one aggregation level.
#[allow(clippy::too_many_arguments)] // all args are genuinely distinct algorithm parameters
fn leiden_recursive(
    graph: &LeidenGraph,
    initial: Vec<usize>,
    rng: &mut StdRng,
    max_iter: usize,
    quality: &QualityFunction,
    gamma: f64,
    min_compression_ratio: f64,
    max_recursion_depth: u32,
    depth: u32,
) -> Vec<usize> {
    if depth >= max_recursion_depth {
        tracing::warn!(
            target: "sdivi_detection::leiden",
            depth = max_recursion_depth,
            "recursion depth cap reached — algorithm halted; check graph health"
        );
        let mut p = initial;
        renumber(&mut p);
        return p;
    }

    let mut partition: Vec<usize> = initial;
    renumber(&mut partition);

    for _iter in 0..max_iter {
        let moved = local_move::local_move_phase(graph, &mut partition, rng, quality, gamma);

        if !moved {
            break;
        }

        let refined = refine_partition(graph, &partition, rng, quality, gamma);

        let AggregateResult {
            graph: agg_graph,
            membership,
        } = aggregate_network(graph, &refined);

        if agg_graph.n >= graph.n {
            break; // identity compression — refinement merged zero nodes
        }
        // Secondary convergence: stop if compression is below the minimum ratio.
        // The float comparison is deterministic given a fixed min_compression_ratio.
        let threshold = graph.n as f64 * (1.0 - min_compression_ratio);
        if agg_graph.n as f64 > threshold {
            tracing::debug!(
                target: "sdivi_detection::leiden",
                depth,
                n_before = graph.n,
                n_after = agg_graph.n,
                "compression cutoff triggered"
            );
            break;
        }

        let agg_init = map_to_aggregate_init(&partition, &refined, agg_graph.n);
        let agg_result = leiden_recursive(
            &agg_graph,
            agg_init,
            rng,
            max_iter,
            quality,
            gamma,
            min_compression_ratio,
            max_recursion_depth,
            depth + 1,
        );

        partition = flatten_partition(&agg_result, &membership, graph.n);
        renumber(&mut partition);
    }

    partition
}

/// Renumbers community IDs to a dense range `[0, k)`, sorted by first-occurrence
/// node index for deterministic output.
fn renumber(partition: &mut [usize]) {
    let mut map = BTreeMap::new();
    let mut next = 0usize;
    for comm in partition.iter_mut() {
        let entry = map.entry(*comm).or_insert_with(|| {
            let c = next;
            next += 1;
            c
        });
        *comm = *entry;
    }
}

/// Converts a flat assignment vector to a [`LeidenPartition`].
fn build_partition(
    _dg: &DependencyGraph,
    graph: &LeidenGraph,
    assignment: Vec<usize>,
    seed: u64,
) -> LeidenPartition {
    let assignments: BTreeMap<usize, usize> = assignment
        .iter()
        .enumerate()
        .map(|(i, &c)| (i, c))
        .collect();

    let stability = compute_stability(graph, &assignment);
    let modularity = compute_modularity(graph, &assignment);

    LeidenPartition {
        assignments,
        stability,
        modularity,
        seed,
    }
}
