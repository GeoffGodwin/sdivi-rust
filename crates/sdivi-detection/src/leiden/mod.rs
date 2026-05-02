//! Native Leiden community detection algorithm.
//!
//! Implements Traag et al. 2019 (arXiv:1810.08473) with Modularity and CPM
//! quality functions.  The algorithm is single-threaded and fully deterministic
//! given the same seed.

mod aggregate;
mod cpm;
pub(crate) mod graph;
mod modularity;
mod quality;
mod refine;

use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;
use rand::SeedableRng as _;
use std::collections::BTreeMap;

use sdivi_graph::DependencyGraph;

use crate::partition::{LeidenConfig, LeidenPartition, QualityFunction};
use crate::warm_start::initial_assignment_from_cache;

use aggregate::{aggregate_network, flatten_partition, map_to_aggregate_init, AggregateResult};
use graph::LeidenGraph;
use modularity::ModularityState;
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
    );
    build_partition(dg, &leiden_graph, assignment, cfg.seed)
}

/// Recursive Leiden step operating on a graph at one aggregation level.
fn leiden_recursive(
    graph: &LeidenGraph,
    initial: Vec<usize>,
    rng: &mut StdRng,
    max_iter: usize,
    quality: &QualityFunction,
    gamma: f64,
) -> Vec<usize> {
    let mut partition: Vec<usize> = initial;
    renumber(&mut partition);

    for _iter in 0..max_iter {
        let moved = local_move_phase(graph, &mut partition, rng, quality, gamma);

        if !moved {
            break;
        }

        let refined = refine_partition(graph, &partition, rng, quality, gamma);

        let AggregateResult {
            graph: agg_graph,
            membership,
        } = aggregate_network(graph, &refined);

        if agg_graph.n >= graph.n {
            break; // No compression — stop.
        }

        let agg_init = map_to_aggregate_init(&partition, &refined, agg_graph.n);
        let agg_result = leiden_recursive(&agg_graph, agg_init, rng, max_iter, quality, gamma);

        partition = flatten_partition(&agg_result, &membership, graph.n);
        renumber(&mut partition);
    }

    partition
}

/// One pass of the Louvain-style local move phase.
///
/// Visits nodes in a random (seed-deterministic) order and moves each node to
/// the neighbour community that maximises the quality function gain.
///
/// Returns `true` if at least one node was moved.
fn local_move_phase(
    graph: &LeidenGraph,
    partition: &mut Vec<usize>,
    rng: &mut StdRng,
    quality: &QualityFunction,
    gamma: f64,
) -> bool {
    let n = graph.n;

    // Offset community IDs by n so they never collide with node indices used as
    // singleton community IDs inside ModularityState::remove_node.  Without the
    // offset, a node at index X that happens to share its ID with an existing
    // community would corrupt that community's size counter when placed in its
    // singleton slot, causing an underflow on a later remove_node call.
    let offset_partition: Vec<usize> = partition.iter().map(|&c| c + n).collect();
    let max_comm = offset_partition.iter().copied().max().unwrap_or(0) + 1;

    let mut state = ModularityState::from_assignment(graph, offset_partition, max_comm);

    let mut order: Vec<usize> = (0..n).collect();
    order.shuffle(rng);

    let mut any_moved = false;

    for &node in &order {
        let old_comm = state.assignment[node];

        // Remove node from its community (updates sigma_tot, size, inner_edges).
        let _k_in_old = state.remove_node(graph, node);

        // Find the best community to move `node` to.
        let best = best_neighbour_community(graph, node, &state, quality, gamma);

        let (best_comm, best_gain) = best;

        // When best_gain > 1e-10, best_comm is always an offset community ID (>= n) and != node.
        // best_comm defaults to `node` (< n) when no neighbour improves the gain; the
        // best_gain threshold below prevents using that default.
        if best_gain > 1e-10 {
            state.add_node(graph, node, best_comm);
            any_moved = true;
        } else {
            // node index is always < n; offset community IDs are >= n, so this singleton ID cannot collide
            let target = if state.size[old_comm] == 0 {
                node
            } else {
                old_comm
            };
            state.add_node(graph, node, target);
        }
    }

    // Write back assignments.
    *partition = state.assignment.clone();
    any_moved
}

/// Finds the neighbour community with the highest positive move gain.
///
/// Returns `(best_community_id, best_gain)`.
fn best_neighbour_community(
    graph: &LeidenGraph,
    node: usize,
    state: &ModularityState,
    quality: &QualityFunction,
    gamma: f64,
) -> (usize, f64) {
    // Accumulate edge weights per candidate community.
    let mut k_in_per_comm: BTreeMap<usize, f64> = BTreeMap::new();
    for (idx, &nbr) in graph.adj[node].iter().enumerate() {
        let c = state.assignment[nbr];
        *k_in_per_comm.entry(c).or_insert(0.0) += graph.edge_weights[node][idx];
    }

    let mut best_comm = node; // default: singleton
    let mut best_gain = 0.0f64;

    for (comm, k_in) in k_in_per_comm {
        let gain = compute_gain(graph, node, comm, k_in, state, quality, gamma);
        if gain > best_gain {
            best_gain = gain;
            best_comm = comm;
        }
    }

    (best_comm, best_gain)
}

/// Computes the move gain for a given quality function.
fn compute_gain(
    graph: &LeidenGraph,
    node: usize,
    comm: usize,
    k_in: f64,
    state: &ModularityState,
    quality: &QualityFunction,
    gamma: f64,
) -> f64 {
    match quality {
        QualityFunction::Modularity => state.move_gain(graph, node, comm, k_in),
        QualityFunction::Cpm { gamma: _ } => {
            let n_comm = state.size[comm] as f64;
            cpm::cpm_move_gain(k_in, n_comm, gamma)
        }
    }
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
