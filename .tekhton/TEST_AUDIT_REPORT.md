## Test Audit Report

### Audit Summary
Tests audited: 2 files, 14 test functions (12 runnable + 1 ignored regression guard + 2 proptest property tests in separate blocks)
Verdict: PASS

### Findings

#### SCOPE: Shell orphan detector reports false positives for both files
- File: `crates/sdivi-detection/tests/refinement.rs`, `crates/sdivi-detection/tests/leiden_termination.rs`
- Issue: The pre-verified orphan list claims both test files "import deleted module '.tekhton/.commit_decision'". Both files were read in full. `refinement.rs` imports `proptest::prelude::*`, `rand::rngs::StdRng`, `rand::SeedableRng`, and four symbols from `sdivi_detection::internal` and `sdivi_graph`. `leiden_termination.rs` imports `sdivi_detection::leiden::run_leiden`, `sdivi_detection::partition::LeidenConfig`, and `sdivi_graph::dependency_graph::build_dependency_graph_from_edges`. A grep for "commit_decision" across both files returns zero matches. `.tekhton/.commit_decision` is not a Rust module path and cannot appear in a `use` statement. These are false positives from the orphan-detection script.
- Severity: LOW
- Action: Dismiss both orphan flags. No test changes needed. Investigate the orphan-detection script's path-matching logic; it is emitting spurious alerts that auditors must manually refute.

#### WEAKENING: Shell weakening detector reports false positive
- File: `crates/sdivi-detection/tests/refinement.rs`
- Issue: The pre-verified weakening report states "net loss of 2 assertion(s) (removed 2, added 0)" in `refinement.rs`. These two assertions (`p.modularity > 0.1` and `p.community_count() >= 2`) were not removed — they were moved verbatim to `leiden_termination.rs:43-52` as part of a line-count compliance refactor documented in CODER_SUMMARY.md. Both assertions are present and exercised in the new file. Net assertion count across both files is unchanged.
- Severity: LOW
- Action: No test change needed. The weakening detector counts deletions from individual files without checking whether deleted content reappears in another file in the same commit. This is a known limitation of single-file diff analysis and should be documented as a known false-positive class for cross-file moves.

#### COVERAGE: Ignored regression guard is intentional and correctly structured
- File: `crates/sdivi-detection/tests/leiden_termination.rs:71-101`
- Issue: `leiden_termination_regression_star_n6_seed0` is marked `#[ignore = "fails until M49.2 fixes leiden blowup; see milestone"]`. This is not a coverage gap — M49.1's stated scope is to capture and contain the hang, not fix it. The ignore annotation documents the responsible milestone (M49.2) and encodes the minimal reproducer minimized by proptest (`n=6, K_{1,5} star, seed=0`). The 30-second thread-join timeout (`rx.recv_timeout(Duration::from_secs(30))`) correctly distinguishes "terminates but wrong" from "does not terminate" once M49.2 un-ignores it.
- Severity: LOW
- Action: None. Confirm M49.2 uses un-ignoring this test as its acceptance criterion.

---

### Rubric Evaluation

**1. Assertion Honesty — PASS**

All assertions derive their expected values from implementation logic or analytically provable invariants:
- `sigma_tot` comparisons use `g.degree[i]` directly, not literals. (`refinement.rs:30-32`)
- The `6.0` in `from_partition_non_singleton_inner_edges` is provable: 3 nodes each with degree 2 in a triangle = 6.0. (`refinement.rs:47`)
- `apply_move_updates_sigma_tot_and_size` checks `sigma_tot[1]` against `g.degree[0] + g.degree[1]` — derived from the actual degree values. (`refinement.rs:67`)
- `well_connected_strong_connection_passes` computes the threshold the same way the implementation does (`gamma * (sc - sc*sc/ss)`) and uses a `1e-9` margin to avoid binary-IEEE-754 representation issues — correctly mirrors `refine.rs:138`. (`refinement.rs:87-88`)
- `prop_refine_modularity_does_not_decrease` computes `q_baseline` via the real `compute_modularity` on the all-singletons assignment — a provably ≤ 0 value — then asserts Leiden improves on it with `1e-9` floating-point tolerance. (`refinement.rs:262-267`)
- The `> 0.1` threshold in `leiden_with_corrected_refine_gives_positive_modularity` is a meaningful lower bound for a ring-of-3-cliques with two dense internal clusters and one bridge edge. (`leiden_termination.rs:43-47`)

**2. Edge Case Coverage — PASS**

- Gamma=0 short-circuit path in `well_connected` is exercised. (`refinement.rs:75-78`)
- Size_s=0 short-circuit path in `well_connected` is exercised. (`refinement.rs:92-97`)
- Singleton and non-singleton `RefinementState` initialization are both tested. (`refinement.rs:21-50`)
- Disconnected subgraphs within one coarse community are tested (should never merge). (`refinement.rs:114-137`)
- Path and clique graphs are both tested for boundary preservation. (`refinement.rs:142-184`)
- Empty edges and zero-total-weight guards in the property test prevent degenerate inputs from producing false failures. (`refinement.rs:253-260`)

**3. Implementation Exercise — PASS**

All tests call real implementation code with no mocking:
- `refine_partition`, `compute_modularity`, `well_connected`, `RefinementState::from_partition`, `RefinementState::apply_move` — called directly via `sdivi_detection::internal` (`lib.rs:34-38`).
- `run_leiden` — called directly via `sdivi_detection::leiden` (`lib.rs:23`).
- `LeidenGraph::from_edges` — real constructor.
- `build_dependency_graph_from_edges` — real function; exercises the full `DependencyGraph` → `LeidenGraph` path that `run_leiden` itself uses (`leiden/mod.rs:50`).

**4. Test Weakening Detection — PASS (false positive resolved; hardening confirmed)**

The proptest split adds `fork: true` + `timeout: 10_000` to `prop_refine_modularity_does_not_decrease` and enables the Cargo features that make those settings non-silent (`Cargo.toml:31`). This is a hardening: cases that previously hung the test binary indefinitely are now killed after 10 seconds and reported as minimized failures. The coarse-community invariant property (`prop_refine_does_not_increase_coarse_communities`) is unchanged. No assertions were removed; no expected values were broadened.

**5. Test Naming and Intent — PASS**

All 14 test names encode both scenario and expected outcome:
- `from_partition_singleton_init_sigma_tot` — input kind, field, condition ✓
- `well_connected_size_s_zero_always_true` — boundary condition named explicitly ✓
- `leiden_termination_regression_star_n6_seed0` — algorithm, bug class, and minimal repro inputs all encoded ✓
- `prop_refine_modularity_does_not_decrease` — property and direction ✓

**6. Scope Alignment — PASS**

All referenced symbols are live in the current codebase:
- `sdivi_detection::internal::{compute_modularity, refine_partition, well_connected, LeidenGraph, RefinementState}` — all re-exported via `lib.rs:34-38`.
- `sdivi_detection::leiden::run_leiden` — public function at `leiden/mod.rs:45`.
- `sdivi_graph::dependency_graph::build_dependency_graph_from_edges` — available via dev-dep at `Cargo.toml:38`.
- `sdivi_parsing::feature_record::FeatureRecord` — available via `sdivi-parsing` dev-dep at `Cargo.toml:39`.
- No references to deleted files, renamed functions, or removed features.

**7. Test Isolation — PASS**

Both test files are fully self-contained:
- All test data is constructed in-memory (`LeidenGraph::from_edges`, `build_dependency_graph_from_edges`, `FeatureRecord` struct literals).
- No test reads from `.tekhton/`, `.sdivi/`, snapshots, build reports, or any other mutable project file.
- No test depends on prior pipeline run state.
- The regression test in `leiden_termination.rs` uses a dedicated thread and `mpsc::channel` — no shared global state, no filesystem I/O.
