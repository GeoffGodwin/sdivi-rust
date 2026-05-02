## Test Audit Report

### Audit Summary
Tests audited: 1 file (`crates/sdivi-detection/tests/refinement.rs`), 13 test functions
(12 authored by coder + 1 added by tester: `well_connected_size_s_zero_always_true`)
Verdict: PASS

---

### Findings

#### NAMING: Test name describes a weaker invariant than what the body checks
- File: `crates/sdivi-detection/tests/refinement.rs:267`
- Issue: `prop_refine_does_not_increase_coarse_communities` implies the test verifies
  that the count of coarse communities does not grow after refinement. The body actually
  checks the subset invariant: for every pair of nodes that share a refined community,
  they must also share a coarse community (`assert_eq!(coarse[i], coarse[j], ...)`).
  The subset invariant is strictly stronger than a count claim. A reader consulting the
  test by name will form an incorrect mental model of what property is covered.
- Severity: LOW
- Action: Rename to `prop_refine_refined_communities_are_subsets_of_coarse_communities`.

#### EXERCISE: Property test evaluates inconsistent graph representations
- File: `crates/sdivi-detection/tests/refinement.rs:285`
- Issue: `prop_refine_modularity_does_not_decrease` constructs two graphs from the same
  raw edge list:
    `g  = LeidenGraph::from_edges(n, &edges)`           — accumulates parallel edge weights
    `dg = build_dependency_graph_from_edges(&paths, &edges)` — deduplicates via `contains_edge`
  The proptest generator produces `(u % n, v % n)` pairs, which can contain duplicates.
  When duplicates occur, `g.edge_weights` for that pair > 1.0 while `dg` stores 1.0.
  Leiden runs on `dg`; both `q_baseline` and `q_leiden` are then computed on `g`. A
  partition that is monotone-improving on `dg` is not guaranteed to be
  monotone-improving when re-evaluated on the differently-weighted `g`. The 256-case
  proptest run passed, but the test is comparing quantities from two structurally
  different graphs without documenting or enforcing the assumption that they coincide.
- Severity: LOW
- Action: Before constructing either graph, deduplicate and sort the edge list
  (`let mut edges = ...; edges.sort(); edges.dedup();`). This makes `g` and `dg`
  topologically identical and the comparison meaningful.

#### COVERAGE: Early-exit guards in `refine_partition` have no targeted tests
- File: `crates/sdivi-detection/tests/refinement.rs` (no test covers these two paths)
- Issue: `refine_partition` (`refine.rs:158`) returns `vec![]` immediately when
  `graph.n == 0`. `refine_community` (`refine.rs:174`) skips coarse communities whose
  `members.len() <= 1`. Every test in the audit set uses graphs with n ≥ 3 and coarse
  communities of size ≥ 2, leaving both guards uncovered.
- Severity: LOW
- Action: Add two unit tests:
  - `refine_empty_graph_returns_empty`: call `refine_partition` on
    `LeidenGraph::from_edges(0, &[])` and assert the result is `vec![]`.
  - `refine_all_singleton_coarse_is_identity`: use partition `(0..n).collect()`
    so every coarse community has exactly one member; assert the refined output
    equals the input (no movement possible within size-1 communities).

---

### Rubric Detail

1. **Assertion Honesty — PASS**
   All numeric literals (`6.0`, `3.0`, `2.0`, `1.0`) are directly derivable from
   the graph topology (node degrees, edge counts) and the `well_connected` formula.
   - `from_partition_non_singleton_inner_edges`: asserts `sigma_tot[0] == 6.0` (3 nodes
     × degree 2 in a K3) and `inner_edges[0] == 3.0` (3 edges in K3). Both correct.
   - `well_connected_*` tests: asserts compare against values computed by the same
     formula as the implementation (`gamma * (sc - sc*sc/ss)`). The tester's
     `threshold + 1e-9` fix is explicitly derived from the implementation formula, not
     hard-coded.
   No always-passing assertions or constants unconnected to implementation logic found.

2. **Edge Case Coverage — PASS**
   - `well_connected_gamma_zero_always_true`: gamma == 0.0 short-circuit
   - `well_connected_size_s_zero_always_true` (tester-added): size_s == 0 short-circuit
   - `well_connected_strong_connection_passes`: k_in above threshold (pass)
   - `well_connected_weak_connection_fails`: k_in below threshold (fail), two sub-cases
   - `two_disconnected_groups_never_mix_after_refine`: no cross-edges between groups
   - `refine_preserves_coarse_community_boundary`: one cross-edge, correct coarse split
   - `refine_path_graph_boundary`: linear topology, weaker inter-community connectivity
   - Property tests cover random n ∈ [3,8] and random edge sets
   Gap: no test for empty graph or single-member coarse community (see COVERAGE finding).

3. **Implementation Exercise — PASS**
   All tests import and call real implementation code:
   `RefinementState::from_partition`, `apply_move`, `move_gain`, `well_connected`,
   `refine_partition`, and the full `run_leiden` pipeline. No mocking of any kind.

4. **Test Weakening Detection — PASS**
   The tester modified one test (`well_connected_strong_connection_passes`) and added
   one test (`well_connected_size_s_zero_always_true`). The modification replaced a
   literal `2.1` boundary comparison (which is not finitely representable in IEEE 754)
   with `threshold + 1e-9` computed from the same formula as the implementation. This
   is a strengthening of floating-point safety, not a weakening of the assertion.
   The added test covers a previously untested early-return branch. No assertion was
   removed or broadened.

5. **Test Naming and Intent — PASS (with NAMING finding above)**
   11 of 13 test names encode both the scenario and the expected outcome. The one
   exception is noted under the NAMING finding. No generic names (`test_1`, etc.).

6. **Scope Alignment — PASS**
   All imported symbols are present in the current implementation:
   - `sdivi_detection::internal::{compute_modularity, refine_partition, well_connected,
     LeidenGraph, RefinementState}` — re-exported at `lib.rs:34-38`
   - `sdivi_detection::leiden::run_leiden` — re-exported at `lib.rs:23`
   - `sdivi_detection::partition::{LeidenConfig, QualityFunction}` — re-exported at `lib.rs:25`
   - `sdivi_graph::dependency_graph::build_dependency_graph_from_edges` — present
   The deleted `.tekhton/test_dedup.fingerprint` is not referenced by any test file.

7. **Test Isolation — PASS**
   All fixture data is constructed in-memory from integer literals, string literals, and
   local `LeidenGraph::from_edges` calls. No test reads `.tekhton/*`, `.sdivi/*`,
   build outputs, snapshot files, or any mutable project state. No filesystem I/O.
