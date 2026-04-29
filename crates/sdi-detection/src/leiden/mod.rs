//! Native Leiden community detection algorithm.
//!
//! Implements Traag et al. 2019 (arXiv:1810.08473) with Modularity and CPM
//! quality functions.  The algorithm is single-threaded and fully deterministic
//! given the same seed.

mod aggregate;
pub(crate) mod graph;
mod modularity;
mod cpm;
mod refine;

use std::collections::BTreeMap;
use rand::SeedableRng as _;
use rand::rngs::StdRng;
use rand::seq::SliceRandom as _;

use sdi_graph::DependencyGraph;

use crate::partition::{LeidenConfig, LeidenPartition, QualityFunction};
use crate::warm_start::initial_assignment_from_cache;

use aggregate::{AggregateResult, aggregate_network, flatten_partition, map_to_aggregate_init};
use graph::LeidenGraph;
use modularity::ModularityState;
use refine::refine_partition;

/// Runs the Leiden algorithm on a [`DependencyGraph`].
///
/// Produces a [`LeidenPartition`] with community assignments, per-community
/// stability scores, and overall modularity.
///
/// # Examples
///
/// ```rust
/// use sdi_detection::leiden::run_leiden;
/// use sdi_detection::partition::{LeidenConfig, LeidenPartition};
/// use sdi_graph::dependency_graph::build_dependency_graph;
/// use sdi_parsing::feature_record::FeatureRecord;
/// use std::path::PathBuf;
///
/// let records: Vec<FeatureRecord> = vec![];
/// let dg = build_dependency_graph(&records);
/// let cfg = LeidenConfig::default();
/// let p = run_leiden(&dg, &cfg, None);
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

        let AggregateResult { graph: agg_graph, membership } =
            aggregate_network(graph, &refined);

        if agg_graph.n >= graph.n {
            break; // No compression — stop.
        }

        let agg_init = map_to_aggregate_init(&partition, &refined, agg_graph.n);
        let agg_result = leiden_recursive(
            &agg_graph,
            agg_init,
            rng,
            max_iter,
            quality,
            gamma,
        );

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
    let max_comm = partition.iter().copied().max().unwrap_or(0) + 1;

    // Build state from current partition.
    let mut state = ModularityState::from_assignment(graph, partition.clone(), max_comm);

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

        if best_comm != node && best_gain > 1e-10 {
            // Move to the better community.
            state.add_node(graph, node, best_comm);
            any_moved = true;
        } else {
            // Stay in (or return to) old community or singleton.
            let target = if state.size[old_comm] == 0 { node } else { old_comm };
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
    // Accumulate edge counts per candidate community.
    let mut k_in_per_comm: BTreeMap<usize, f64> = BTreeMap::new();
    for &nbr in &graph.adj[node] {
        let c = state.assignment[nbr];
        *k_in_per_comm.entry(c).or_insert(0.0) += 1.0;
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
        QualityFunction::Modularity => {
            state.move_gain(graph, node, comm, k_in)
        }
        QualityFunction::Cpm { gamma: _ } => {
            let n_comm = state.size[comm] as f64;
            cpm::cpm_move_gain(k_in, n_comm, gamma)
        }
    }
}

/// Renumbers community IDs to a dense range `[0, k)`, sorted by first-occurrence
/// node index for deterministic output.
fn renumber(partition: &mut Vec<usize>) {
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

    LeidenPartition { assignments, stability, modularity, seed }
}

/// Computes per-community stability (internal edge density).
fn compute_stability(graph: &LeidenGraph, assignment: &[usize]) -> BTreeMap<usize, f64> {
    let max_comm = assignment.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut size = vec![0usize; max_comm];
    let mut inner = vec![0.0f64; max_comm];

    for (i, &c) in assignment.iter().enumerate() {
        size[c] += 1;
        for &j in &graph.adj[i] {
            if assignment[j] == c && j > i {
                inner[c] += 1.0;
            }
        }
    }

    let mut stability = BTreeMap::new();
    for c in 0..max_comm {
        if size[c] == 0 {
            continue;
        }
        let n = size[c] as f64;
        let max_possible = n * (n - 1.0) / 2.0;
        let s = if max_possible > 0.0 { inner[c] / max_possible } else { 1.0 };
        stability.insert(c, s);
    }
    stability
}

/// Computes overall modularity.
fn compute_modularity(graph: &LeidenGraph, assignment: &[usize]) -> f64 {
    let m = graph.total_weight;
    if m == 0.0 {
        return 0.0;
    }
    let max_comm = assignment.iter().copied().max().map(|m| m + 1).unwrap_or(0);
    let mut sigma = vec![0.0f64; max_comm];
    let mut inner = vec![0.0f64; max_comm];

    for (i, &c) in assignment.iter().enumerate() {
        sigma[c] += graph.degree[i];
        for &j in &graph.adj[i] {
            if assignment[j] == c && j > i {
                inner[c] += 1.0;
            }
        }
    }

    let m2 = 2.0 * m;
    sigma.iter().zip(inner.iter()).fold(0.0, |acc, (&s, &l)| {
        acc + l / m - (s / m2).powi(2)
    })
}
