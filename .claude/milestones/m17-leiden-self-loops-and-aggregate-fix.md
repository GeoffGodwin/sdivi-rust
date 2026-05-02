
#### Milestone 17: LeidenGraph Self-Loops + Aggregate Correctness

<!-- milestone-meta
id: "17"
status: "done"
-->

**Scope:** Fix two correctness bugs in the recursive Leiden machinery that together produce the all-one-community collapse currently observed on the verify-leiden fixtures (`small_fixture_quality` returns `modularity = 0.0` against the leidenalg reference of `0.778`):

1. `aggregate_network` drops intra-community edges. The Leiden paper requires aggregate super-nodes to carry **self-loops** whose weight equals the sum of the original intra-community edge weights. Without them, the aggregated graph forgets the strong intra-community pull, and the optimal partition at the aggregate level becomes "merge everything."
2. `aggregate_network` double-counts inter-community edges (each undirected edge is summed twice — once per endpoint).

Fixing the aggregate requires `LeidenGraph` to support self-loops as a first-class concept (currently `from_edges_weighted` silently drops them via `if u == v { continue; }`). This milestone is the data-structure half of the proper Leiden fix; M18 follows with the refinement-phase rewrite.

**Why this milestone exists:** The `verify-leiden` cross-check suite exists to enforce KDD-2's *partition-quality* contract (modularity within 1% of leidenalg, community count within ±10%). It's been silently failing or hitting the 6-hour CI default since M05 ratification because no other Leiden test asserts specific modularity values — the existing tests in `partition.rs`, `proptest_seeded.rs`, `weighted_edges.rs`, and `leiden_id_collision.rs` only check structural properties (`community_count() >= 1`, JSON round-trip, deterministic seed, weighted ≥ unweighted). Probing on the small fixture revealed all 50 nodes collapse into a single community — the canonical signature of a broken aggregate phase. KDD-2 says partition quality is the contract, and the contract is currently violated.

**Theoretical basis:** Traag, Waltman, & van Eck (2019), *From Louvain to Leiden: guaranteeing well-connected communities*, Section "Algorithm" (esp. the aggregation step), and the convention used by igraph / leidenalg: an undirected self-loop of weight `w` on node `u` contributes `2w` to `degree[u]` (both endpoints are `u`) and `w` once to `total_weight`. In modularity `Q = Σ_c [L_c / m − (Σ_c / 2m)²]`, self-loops of nodes in `c` contribute their weight to `L_c` directly. ΔQ for moving a node `v` to community `c` is **independent** of `v`'s self-loop weight (the loop is internal to `v`'s community by definition both before and after the move) — see derivation in `Watch For` below.

**Deliverables:**

- **Self-loop field on `LeidenGraph`:**
  - Add `self_loops: Vec<f64>` to `LeidenGraph` (length `n`, default `0.0`). `self_loops[u]` is the total weight of all self-loop edges on node `u` (typically 0 for non-aggregate graphs; non-zero in aggregates representing collapsed intra-community weight).
  - Update the `Debug`/`Clone` derives so the new field is included.
  - Update existing constructors to initialise `self_loops` to `vec![0.0; n]`.

- **`from_edges_weighted` accepts self-loops:**
  - Change the `if u == v || u >= n || v >= n { continue; }` guard to skip only out-of-range indices: `if u >= n || v >= n { continue; }`.
  - Self-loops `(u, u, w)`: accumulate `self_loops[u] += w`. Do **not** add to `adj[u]` or `edge_weights[u]` (self-loops are not stored as adjacency entries — the `adj` array is for cross-edges only, which keeps the inner loops in `local_move_phase` and `compute_modularity` simple).
  - `degree[u]` calculation: keep the existing `edge_weights[u].iter().sum()` for cross-edges, then add `2.0 * self_loops[u]`. Document the `2×` factor with a comment referencing the standard undirected-graph convention.
  - `total_weight` calculation: change from `degree.iter().sum::<f64>() / 2.0` to `cross_edge_total + self_loops.iter().sum::<f64>()`, where `cross_edge_total` is the sum of weights in `weight_acc` (cross-edge accumulator, post de-dup). Equivalent re-expression: `(degree.iter().sum() - self_loops.iter().sum()) / 2.0 + self_loops.iter().sum()` simplified to `degree.iter().sum::<f64>() / 2.0` only happens to give the right answer because the `2 × self_loops` in degree exactly compensates — verify with a doctest on a 1-node 1-self-loop graph.
  - The `from_edges` (unweighted) constructor must also stop dropping self-loops; pass them through as weight `1.0`.

- **`edge_weight(u, v)` consistency:**
  - When `u == v`, return `self_loops[u]`. Currently `binary_search(&v)` against `adj[u]` returns `Err(_)` and yields `0.0`, which is wrong once self-loops exist.

- **`aggregate_network` correctness fix:**
  - Iterate edges by traversing only the upper triangle (`for u in 0..n { for &v in &graph.adj[u] { if v < u { continue; } ... } }`). This processes each undirected cross-edge exactly **once**, eliminating the current 2× double-count.
  - Per visited cross-edge `(u, v, w)`:
    - If `cu == cv` (intra-community, where `cu = refined_partition[u]`): emit a self-loop `(cu, cu, w)` on the aggregate. **Do not skip.**
    - If `cu != cv` (inter-community): emit a single cross-edge with weight `w` between `(min(cu,cv), max(cu,cv))`.
  - Per visited self-loop `self_loops[u]`: emit a self-loop on `cu` of weight `self_loops[u]`. (Self-loops on the source graph aggregate into self-loops on the target, regardless of the partition.)
  - Build the aggregate via `LeidenGraph::from_edges_weighted` passing the (now possibly self-loop-bearing) edge list. The constructor handles the rest.

- **`compute_modularity` and `compute_stability` honour self-loops:**
  - In `compute_modularity` (`quality.rs`): after the existing `for (i, &c)` loop that walks `adj[i]` for `j > i`, add `inner[c] += graph.self_loops[i]` per node. This makes `L_c` include self-loop weight on community members.
  - In `compute_stability`: same addition. Stability is internal-edge density per community; self-loops count as internal.
  - The `max_possible` divisor in `compute_stability` (`n * (n - 1) / 2.0`) is for non-self-loop pairs; self-loops add to the numerator without expanding the denominator. Document this in a doctest example for a 1-node graph with a self-loop (`max_possible = 0`, special-cased to `1.0` already).

- **`ModularityState` honours self-loops:**
  - `from_assignment`: after the cross-edge loop, add `inner_edges[c] += graph.self_loops[i]` per node.
  - `remove_node`: after the existing cross-edge `inner_edges[comm] -= w` loop, subtract `graph.self_loops[node]` from `inner_edges[comm]` (the self-loop leaves the community when `node` becomes a singleton at slot `node`). Conversely, the new singleton at slot `node` should get the self-loop: set `inner_edges[node] = graph.self_loops[node]` (currently `inner_edges[node]` is left at whatever it was, which is 0 for the singleton case but should be made explicit and self-loop-aware).
  - `add_node`: after the cross-edge `inner_edges[to] += w` loop, add `graph.self_loops[node]` to `inner_edges[to]`.
  - `move_gain` is **unchanged**. The `k_in_to - k_v * sigma_to / m2` formula is the simplified ΔQ for moving from a singleton to community `to`; the self-loop contribution to `L` cancels exactly in the difference (see Watch For for derivation). The `k_v` already includes `2 × self_loops[node]` via `graph.degree`.

- **Unit tests on the self-loop algebra:**
  - `crates/sdivi-detection/src/leiden/graph.rs` inline test: a 1-node graph with one self-loop of weight `2.0` has `degree[0] = 4.0`, `total_weight = 2.0`, `self_loops[0] = 2.0`, `adj[0] = []`.
  - A 2-node graph with one cross-edge of weight `1.0` and self-loops `(0, 0, 0.5)` and `(1, 1, 0.5)` has `degree = [2.0, 2.0]`, `total_weight = 2.0`, modularity for partition `[0, 1]` (each node alone): `Q = 0.5/2 - (2/4)^2 + 0.5/2 - (2/4)^2 = 0.25 - 0.25 + 0.25 - 0.25 = 0.0`; for partition `[0, 0]` (both in same community): `Q = (1.0 + 0.5 + 0.5)/2 - (4/4)^2 = 1.0 - 1.0 = 0.0`. Both give `Q = 0` — the graph has no community structure beyond the self-loops. (Hand-derived; encode in the test as the canonical self-loop sanity check.)
  - A 2-node graph with one cross-edge of weight `1.0` and **no** self-loops has `Q[0,0] = 1/1 - (2/2)^2 = 0`, `Q[0,1] = 0/1 - (1/2)^2 + 0/1 - (1/2)^2 = -0.5`. Confirms the modularity formula matches the no-self-loop baseline (regression check).
  - Aggregate test: a 4-node graph with two cliques `{0,1}` and `{2,3}`, all internal edges weight `1.0`, no cross-edges. Partition `[0, 0, 1, 1]`. After `aggregate_network`, the aggregate must be a 2-node graph with `self_loops = [1.0, 1.0]`, `adj = [[], []]` (no cross-edges), `degree = [2.0, 2.0]`, `total_weight = 2.0`. Modularity of `[0, 1]` on this aggregate must equal modularity of `[0, 0, 1, 1]` on the original — this is the **invariance-under-aggregation** property the Leiden paper relies on.
  - Aggregate test (with cross-edges): a 4-node graph with cliques as above plus a cross-edge `(1, 2, 1.0)`. Partition `[0, 0, 1, 1]`. Aggregate must be a 2-node graph with `self_loops = [1.0, 1.0]`, one cross-edge `(0, 1, 1.0)` (single, not doubled), `total_weight = 3.0`. Modularity invariance must still hold.

- **Doctest updates:**
  - `LeidenGraph::from_edges_weighted` rustdoc: extend the Examples block to show a self-loop being preserved.
  - `compute_modularity` rustdoc: add a self-loop example.
  - `aggregate_network` does not currently have a public rustdoc example (it's `pub(crate)`); leave as-is.

- **CHANGELOG.md** entry: "Leiden algorithm: `LeidenGraph` now supports self-loops (`self_loops` field). `aggregate_network` correctly preserves intra-community weight as self-loops on the aggregate super-nodes and no longer double-counts inter-community edges. Per-step correctness verified by new aggregate-invariance unit tests; full algorithm correctness gated by M18."

**Migration Impact:** `LeidenGraph` gains a public field `self_loops: Vec<f64>` (the type is `pub(crate)`, so this is internal API). `Snapshot` schema is unchanged (`snapshot_version` stays `"1.0"`). Modularity values for graphs without self-loops are unchanged. Modularity values for *aggregate* graphs are now correct — but aggregate graphs are entirely internal to the Leiden recursive call and not exposed to consumers.

**Files to create or modify:**

- **Modify:** `crates/sdivi-detection/src/leiden/graph.rs` — add `self_loops` field, fix `from_edges_weighted` and `from_edges`, update `degree`/`total_weight`/`edge_weight`, add inline tests.
- **Modify:** `crates/sdivi-detection/src/leiden/aggregate.rs` — rewrite the inner loop to walk the upper triangle, emit self-loops on intra-community visits, dedupe cross-edges. Add doctests where the function becomes `pub` for testing purposes (or expose helpers via `#[cfg(test)]`).
- **Modify:** `crates/sdivi-detection/src/leiden/modularity.rs` — `from_assignment`, `remove_node`, `add_node` honour `self_loops`. `move_gain` is **not** changed.
- **Modify:** `crates/sdivi-detection/src/leiden/quality.rs` — `compute_modularity`, `compute_stability` honour `self_loops`.
- **Modify:** `crates/sdivi-detection/src/leiden/mod.rs` — no logic changes; verify `local_move_phase`'s gain math (already in `move_gain`) does not need updates.
- **New tests:** add a dedicated `crates/sdivi-detection/tests/aggregate_invariance.rs` with the four hand-derived test cases above plus a property test (`prop_aggregate_modularity_invariance`) that asserts: for any graph and any partition, `compute_modularity(graph, partition) ≈ compute_modularity(aggregate(graph, partition), [0..k])` within `1e-9` tolerance.
- **Modify:** `CHANGELOG.md`.

**Acceptance criteria:**

- `cargo test -p sdivi-detection` passes (existing tests + new unit/integration tests). No `cargo test --features verify-leiden` requirement at this milestone — the verify-leiden suite is gated by M18.
- The four hand-derived self-loop test cases pass.
- The property test `prop_aggregate_modularity_invariance` passes with at least 256 cases.
- `cargo clippy --workspace --exclude sdivi-wasm --exclude sdivi-rust -- -D warnings` passes.
- `cargo fmt --check` passes.
- `cargo doc --workspace --exclude sdivi-wasm --exclude sdivi-rust --no-deps` passes with `RUSTDOCFLAGS=-D warnings`.
- `Snapshot` JSON output for the simple-rust fixture is byte-identical before and after this milestone (the simple-rust fixture has no aggregation step in its current code path because the graph is small enough that `agg_graph.n >= graph.n` triggers the early break — so this is a regression check that the *base* graph behaviour is preserved).
- `bindings/sdivi-wasm` builds for `wasm32-unknown-unknown` and the existing wasm smoke tests still pass (`Self_loops` is internal to sdivi-detection; nothing crosses the WASM boundary).

**Tests:**

- Unit (graph.rs): self-loop accumulation, degree/total_weight arithmetic, `edge_weight(u, u)` returns self-loop weight.
- Unit (modularity): self-loop contribution to `L_c`, `remove_node`/`add_node` self-loop bookkeeping.
- Unit (aggregate): four hand-derived cases above.
- Property (aggregate_invariance): random graphs + random partitions → modularity invariance under aggregation.
- Regression: existing `partition.rs`, `proptest_seeded.rs`, `weighted_edges.rs`, `leiden_id_collision.rs`, `warm_start.rs` all pass (these are the M05 baseline tests).
- Doctest: 1-node graph with self-loop, 2-node graph with cross-edge + self-loops.

**Watch For:**

- **The 2× degree convention for self-loops.** A self-loop of weight `w` adds `2w` to `degree[u]`, not `w`. This is the standard undirected-graph convention (both endpoints of the loop are `u`, so each endpoint contributes `w`). It's consistent with `igraph`/`leidenalg`. Pin a doctest verifying `degree[u] == 2 * self_loops[u]` on a node with no cross-edges.
- **Total weight does NOT double-count self-loops.** A self-loop of weight `w` adds `w` to `total_weight`, not `2w`. The cross-edge convention is `total_weight += w` per undirected edge (each cross-edge of weight `w` adds `2w` to total degree, hence `total_weight = total_degree / 2`). For a self-loop, total degree += `2w` and the loop is one undirected edge of weight `w`, so total_weight += `w` is consistent with `total_weight = total_degree / 2` only if you're careful about the counting. The cleanest formulation: `total_weight = sum_of_cross_edge_weights + sum_of_self_loop_weights`. Encode this directly; do not rely on the `degree.sum() / 2` shortcut once self-loops exist.
- **`move_gain` self-loop derivation.** For a node `v` with self-loop weight `s_v` moving from singleton `{v}` to community `c`: before move, `L_{v} = s_v`, `σ_{v} = k_v` (where `k_v` includes `2 s_v`); after move, `L_{c+v} = L_c + k_v_to_c + s_v`, `σ_{c+v} = σ_c + k_v`. The ΔQ algebra gives `ΔQ = k_v_to_c / m − σ_c · k_v / (2m²)` — the `s_v / m` from `L_{c+v}` cancels with the `s_v / m` from removing v's old singleton's `L`. This is why `move_gain` does not need a self-loop term. Hand-verify on the 2-node-with-self-loops test case.
- **`from_assignment` initial inner_edges for the offset partition.** When `local_move_phase` initialises state with `offset_partition` (each node in its own offset community of size 1), `inner_edges[offset_c]` should equal `self_loops[node]` — the singleton's only internal edges are its self-loops. The existing code currently leaves it at 0 (correct when self_loops are 0). After this milestone the initial value must include the self-loop. Verify on an aggregate-graph local-move pass: super-node X with self-loop weight `5.0` placed alone in its singleton community has `inner_edges[c] = 5.0`, not `0.0`.
- **`local_move_phase`'s `add_node` after the no-move branch.** When `best_gain <= 1e-10`, the node is added back to `old_comm` (or to slot `node` if `old_comm` is now empty). `add_node` will correctly add the self-loop weight back into `inner_edges[old_comm]`, undoing the `remove_node` that subtracted it. Verify with a no-move scenario in a unit test.
- **`renumber` does not touch `self_loops`.** `renumber` re-labels community IDs. Self-loops live on nodes, not on communities, so they're independent of community ID. No code change needed; just don't accidentally key self-loops by community ID anywhere.
- **Parallel-edge accumulation in `from_edges_weighted`.** The existing accumulator `weight_acc[(u,v)] += w` collapses parallel edges (same `(u, v)` appearing multiple times in the input). Self-loops at `(u, u)` should also accumulate: a graph created from `[(0, 0, 1.0), (0, 0, 2.0)]` has `self_loops[0] = 3.0`. Test this.
- **Empty graph (`n = 0`).** `self_loops = vec![]`, `total_weight = 0`. The existing `if leiden_graph.n == 0` short-circuit in `run_leiden` continues to handle this.
- **Aggregate of a graph with self-loops.** When aggregating an already-aggregated graph (deeper recursion levels), source self-loops `self_loops[u]` must transfer to the aggregate as self-loops on `cu` regardless of the partition. Test this with a 2-level aggregate scenario.
- **Hash-stability of `from_edges_weighted` output.** `weight_acc` is a `BTreeMap`, so iteration order is deterministic — adjacency lists are sorted by neighbour ID. Self-loops do **not** appear in `adj` so they don't affect iteration order. But the choice of where to insert the self-loop accumulation (before or after the cross-edge loop) must be deterministic; doing it inside the same `for &(u, v, w) in edges` loop with `if u == v` keeps it deterministic and avoids a separate pass.
- **Doc-comment placement when adding the `self_loops` field.** Insert a `///` block ending in a blank line before the next field's `///` block, otherwise rustdoc reattaches the doc to the wrong field (CLAUDE.md "Doc comment placement when inserting items" warning).

**Seeds Forward:**

- M18 builds on the corrected `aggregate_network` and self-loop-aware modularity. With M17's fixes, the aggregate phase is structurally sound; M18 only needs to fix the refinement phase (`refine_partition`) for the full Leiden algorithm to converge.
- The `LeidenGraph.self_loops` field is internal (`pub(crate)`) and is not part of any exported snapshot or WASM API. It can evolve freely until proper Leiden is shipped.
- The `prop_aggregate_modularity_invariance` test stays in the suite as a permanent regression gate. Any future change to aggregation logic must keep modularity invariant under aggregation.
- Once both M17 and M18 land, the verify-leiden CI workflow can be restored to push/PR triggers and the 30-min job timeout remains as a safety net (the actual run on the large fixture should be <2 min in debug mode after the fixes).

---
