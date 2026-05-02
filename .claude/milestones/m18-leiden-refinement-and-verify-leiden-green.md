
#### Milestone 18: Leiden Refinement Phase + verify-leiden Green

<!-- milestone-meta
id: "18"
status: "done"
-->

**Scope:** Replace the broken `refine_partition` implementation with a correct Leiden refinement phase per Traag, Waltman, & van Eck (2019), so the full recursive Leiden algorithm produces partition-quality output that satisfies the `verify-leiden` cross-check suite (modularity within 1% of leidenalg, community count within ±10%) on all three fixtures (small/medium/large). With M17's self-loop and aggregate fixes already in place, this milestone closes the algorithm correctness gap and re-enables `verify-leiden.yml` on push/PR triggers.

**Why this milestone exists:** The current `refine.rs:151` `best_candidate` function uses a **fake** sigma_tot — it computes `count of v's neighbours in sub-community comm` and labels it "approximate sigma_tot for sub-comm." That value is essentially `k_in_to` itself, so the gain formula reduces to roughly `k_in − degree·k_in/2m`, which is positive for almost every candidate. The refinement phase therefore shuffles nodes randomly between sub-communities for `max_iter = 10` iterations and produces a near-random refined partition. Combined with the (now-fixed by M17) aggregate bugs, this caused all 50 nodes of the small fixture to collapse into one community at modularity 0.0. With M17's aggregate corrections in place, the local-move phase produces a reasonable coarse partition, but the broken refinement still poisons the aggregate-graph construction and the recursive call. Fixing refinement is the last piece needed for Leiden to converge to a high-quality partition.

**Theoretical basis:** Traag et al. 2019, *From Louvain to Leiden: guaranteeing well-connected communities*, Algorithm 2 (Refinement). Refinement processes each node within its **coarse** community, attempting to merge it into a neighbouring sub-community **inside the same coarse community**. The novel constraint relative to Louvain is that only "well-connected" sub-communities can absorb a node — this is what gives Leiden its quality guarantee that every community is internally well-connected. For v0 we adopt a simplified refinement that:
1. Tracks real per-sub-community `Σ_tot` (not a count fudge).
2. Uses argmax over positive ΔQ, candidates restricted to neighbour sub-communities within the same coarse community.
3. Optionally gates moves by a γ-connectivity threshold.

The simplified version is sufficient for partition quality on the verify-leiden fixtures (the leidenalg reference is itself an approximation; we only need to be within 1% modularity). Faithful Traag-2019 random selection (probability proportional to `exp(ΔQ/θ)`) is filed as a future polish — it changes which local optima are reached but not whether quality is high.

**Deliverables:**

- **Rewrite `refine.rs::best_candidate` to use real sigma_tot:**
  - Accept a `&RefinementState` (new struct, defined below) instead of `&[usize] refined`.
  - For each candidate sub-community `comm`, look up `state.sigma_tot[comm]` (the sum of degrees of nodes currently in `comm`, **including the 2× self-loop contribution**).
  - Gain formula: `k_in_to - degree[node] * sigma_tot[comm] / (2 * total_weight)` — the same simplified ΔQ formula used in `modularity::ModularityState::move_gain`. **No more "approximate" fudge.**
  - Self-loops on `node` cancel in the gain (per the M17 derivation), so the formula needs no self-loop term.

- **`RefinementState` struct (mirrors `ModularityState`, scoped to refinement):**
  - Fields: `assignment: Vec<usize>`, `sigma_tot: Vec<f64>`, `inner_edges: Vec<f64>`, `size: Vec<usize>`. Same shape as `ModularityState`.
  - Constructor `from_partition(graph: &LeidenGraph, partition: &[usize], capacity: usize) -> RefinementState`. The initial `partition` for refinement is each node in its own singleton (`(0..n).collect()`), so each slot has `size = 1`, `sigma_tot = degree[i]`, `inner_edges = self_loops[i]` (the singleton's only internal edge is its self-loop).
  - `apply_move(graph: &LeidenGraph, node: usize, from: usize, to: usize)`: bookkeeping when moving `node` from sub-community `from` to sub-community `to`. Updates `assignment`, `size`, `sigma_tot`, `inner_edges` (cross-edges and self-loop) for both communities.
  - Place in `refine.rs` as `pub(crate) struct RefinementState` to keep refinement-specific bookkeeping local. Do **not** reuse `ModularityState` directly because refinement's `from_partition` constructor differs (singleton-init) and conflating the two would risk later bugs.

- **`refine_community` rewritten:**
  - Initialise `RefinementState` with each member node as its own singleton sub-community.
  - Iterate up to `max_iter = 10` passes (preserve the existing cap; refinement is meant to be quick).
  - Per pass, shuffle `members` with `rng.shuffle()` (unchanged).
  - For each `node` in shuffled order:
    - Compute `current_comm = state.assignment[node]`.
    - Build candidate set: sub-communities of neighbours within the *same coarse community* (`member_set.contains(nbr)`) that are different from `current_comm`. Same as today.
    - For each candidate, compute the real ΔQ via `state.move_gain(graph, node, candidate, k_in_to)` — analogous to `ModularityState::move_gain`.
    - Pick the argmax candidate with `gain > 1e-10`. Apply the move via `state.apply_move`.
  - If no node moved in a pass, break early.
  - At the end of `refine_community`, write `state.assignment[member]` into the shared `refined: &mut [usize]` for each `member`.

- **`refine_partition` orchestration unchanged in spirit:**
  - Keep the outer "group by coarse community → call `refine_community` per group" structure.
  - Initial `refined = (0..graph.n).collect()` (each node in its own singleton, globally).
  - After each `refine_community` writes back, the global `refined` vector reflects the per-coarse-community sub-partitioning.
  - End with `renumber_in_place(&mut refined)` (unchanged) to compress IDs to `[0, k)`.

- **Optional γ-connectivity gate (feature-flagged for v0, default off):**
  - Add a function `well_connected(graph, node, candidate, &state, gamma) -> bool` that checks whether moving `node` into `candidate` keeps the sub-community γ-connected per Traag 2019:
    - `E(candidate) ≥ γ · |candidate| · (|S| − |candidate|) / (2 m_S)` where `S` is the coarse community.
    - For v0 we compute a simpler proxy: `k_in_to >= gamma * (size_candidate * 1.0 - size_candidate * size_candidate / size_S)` — the formula is documented in the rustdoc with a note that it's the "v0 simplification." 
  - Gate filtered out when `gamma = 0.0` (the verify-leiden default config has `gamma = 1.0`, so this is informational).
  - Mark `well_connected` as `pub(crate) fn` so a future milestone can swap in the full Traag formulation. Not gated behind a Cargo feature — runtime parameterised by `gamma`.

- **`leiden_recursive` integration verified:**
  - No code changes in `mod.rs` are expected. The `for _iter in 0..max_iter` loop already calls `refine_partition` then `aggregate_network` then recurses. With M17's correct aggregate and this milestone's correct refine, the recursion should converge.
  - Add a debug-only assertion (`#[cfg(debug_assertions)]`) that `aggregate_network`'s output graph has fewer nodes than its input (`agg.n < graph.n`) **except** when refinement returned the trivial identity partition. The existing `if agg_graph.n >= graph.n { break; }` early break handles the identity case; the assert documents the invariant.

- **`verify-leiden.yml` re-enable triggers:**
  - Restore `on:` to `push: [main, "milestones/**", ...]` and `pull_request: branches: [main]` paths-filtered as before. The existing `paths:` filter (`crates/sdivi-detection/**`, `tests/fixtures/leiden-graphs/**`, `tools/generate-leiden-fixtures.py`) stays.
  - Keep the `timeout-minutes: 30` job-level cap and the per-step caps (`5/5/15`) added in the billing-incident response.
  - The `cargo test --features verify-leiden` step continues to run in **debug** mode (no `--release`). Per the Watch For below, the small fixture should now run in <30s in debug; medium <60s; large <300s. All three fit comfortably within the 15-min step cap.

- **Test fixture verification:**
  - All three `verify-leiden` fixtures (`small`, `medium`, `large`) pass:
    - `[small] modularity within 1% of 0.778261` and `community_count within ±10% of 5`.
    - `[medium] modularity within 1% of 0.899...` and `community_count within ±10% of 10`.
    - `[large] modularity within 1% of 0.979...` and `community_count within ±10% of 50`.
  - The "no community larger than 50% of node count" assertion in `leiden_quality.rs:103` continues to hold.

- **CHANGELOG.md** entry: "Leiden refinement phase rewritten to use real per-sub-community Σ_tot (replacing the previous count-of-neighbours fudge that caused the all-one-community collapse). All three verify-leiden fixtures (small/medium/large) now pass within 1% modularity tolerance against leidenalg. The verify-leiden CI workflow is re-enabled on push/PR with a 30-min job-level timeout safeguard."

- **DRIFT_LOG.md** entry: "M17 + M18 closed the Leiden correctness regression that was hidden by the absence of a modularity-asserting test outside `verify-leiden.yml`. Pre-M17 partition tests verified `community_count() >= 1` and structural properties only, missing the modularity=0 collapse. New regression gates: `prop_aggregate_modularity_invariance` (M17) and the verify-leiden suite (M18, now CI-blocking)."

**Migration Impact:** None. `LeidenPartition` JSON shape is unchanged. `Snapshot` schema stays `"1.0"`. Existing snapshots remain valid. Modularity values for new snapshots will *change* relative to pre-M18 snapshots (since the previous algorithm produced incorrect partitions), but no consumer should have been relying on the broken modularity values; the change-coupling-aware Leiden in M15 was riding on top of the same broken core. Document in CHANGELOG that snapshot deltas across the M16 → M18 cutover may show artificial "drift" purely from the algorithm correction; recommend that adopters either (a) compare snapshots only within the M18-or-later era, or (b) re-baseline at the M18 boundary.

**Files to create or modify:**

- **Modify:** `crates/sdivi-detection/src/leiden/refine.rs` — rewrite `best_candidate`, `refine_community`, `try_merge_node`. Add `RefinementState` struct. Add `well_connected` helper.
- **Modify:** `crates/sdivi-detection/src/leiden/mod.rs` — optional debug assertion in `leiden_recursive`. No structural changes.
- **Modify:** `.github/workflows/verify-leiden.yml` — restore `push:` and `pull_request:` triggers (currently set to `workflow_dispatch:` only? — verify by reading the file; if the user already restored them, leave as-is).
- **Modify:** `CHANGELOG.md`, `.tekhton/DRIFT_LOG.md`.
- **New tests:**
  - `crates/sdivi-detection/tests/refinement.rs` — hand-built coarse partitions on small graphs where the correct refinement is hand-computable. Three cases:
    1. Two cliques merged into one coarse community → refinement must split them back out (partition `[0, 0, 0, 0, 0, 0]` with two triangle cliques `{0,1,2}` and `{3,4,5}` connected only by no edge → refine returns `[0,0,0,1,1,1]` or equivalent).
    2. A single clique as one coarse community → refinement should keep it as one sub-community (merging into the singleton-of-the-first-shuffled-node).
    3. A path graph (chain) as one coarse community → refinement output depends on shuffle but must give modularity-positive sub-communities.
  - The existing `crates/sdivi-detection/tests/leiden_quality.rs` is the de-facto integration test for full algorithm correctness. No changes needed; it just starts passing.

**Acceptance criteria:**

- `cargo test -p sdivi-detection --features verify-leiden` passes locally on rustc 1.85, debug mode, in under 5 minutes wall-clock (small <30s, medium <60s, large <300s; numbers chosen with 5× headroom over typical observed runtime).
- `cargo test -p sdivi-detection --features verify-leiden --release` passes in under 30 seconds total.
- `cargo test --workspace --exclude sdivi-wasm --exclude sdivi-rust` passes (all existing tests, including M17's new aggregate-invariance suite, continue to pass).
- The `verify-leiden.yml` CI workflow runs successfully on push to milestones/v0 and on pull_request to main, completing within the 15-minute step cap.
- `cargo clippy --workspace --exclude sdivi-wasm --exclude sdivi-rust -- -D warnings` passes.
- `cargo fmt --check` passes.
- `cargo doc --workspace --exclude sdivi-wasm --exclude sdivi-rust --no-deps` passes with `RUSTDOCFLAGS=-D warnings`.
- `bindings/sdivi-wasm` continues to compile for `wasm32-unknown-unknown` and the existing wasm smoke tests pass (refinement is internal to sdivi-detection; no WASM surface change).
- The `partition.modularity` field of a `LeidenPartition` for the small fixture is `≥ 0.770` (within 1% below leidenalg's 0.778261, allowing for the standard tolerance band).
- The "no community larger than 50% of node count" invariant from `leiden_quality.rs:103` continues to hold on all three fixtures (the all-one-community collapse exposed in this debugging session is a hard violation; M18 must eliminate it).

**Tests:**

- Unit (refine.rs): `RefinementState::from_partition` initial state for a 3-node singleton-init graph. `apply_move` correctly updates `sigma_tot`, `inner_edges`, `size`, `assignment`. `well_connected` accepts/rejects per a hand-derived γ-threshold case.
- Unit (refine.rs): `best_candidate` returns the highest-gain candidate from a known set; ties broken by smallest community ID (BTreeMap iteration order, deterministic).
- Integration (refinement.rs): three hand-built coarse partitions with hand-computed expected refined partitions. Each test asserts `refined` matches one of the expected outputs (use a permutation-equivalence helper to allow IDs to be renumbered).
- Property (proptest): `prop_refine_does_not_increase_coarse_communities` — for any graph and any coarse partition, `refine_partition` returns a refined partition where every refined community is a subset of some coarse community. Encode via `for n in 0..graph.n: coarse_partition[n] same for all n in same refined community`.
- Property (proptest): `prop_refine_modularity_does_not_decrease` — running `local_move_phase`, `refine_partition`, `aggregate_network`, recursive `local_move_phase` on the aggregate, and flattening back must give a modularity `>= the modularity after the first local_move_phase`. (Leiden monotone-improvement guarantee.)
- Integration (leiden_quality.rs, gated): all three fixtures pass within tolerance.
- Determinism (proptest_seeded.rs): same seed → bit-identical `LeidenPartition` JSON across 100 runs (existing test, must continue to pass after refinement rewrite).
- Regression: `partition.rs`, `weighted_edges.rs`, `leiden_id_collision.rs`, `warm_start.rs` all pass unchanged.

**Watch For:**

- **Refinement starts from singletons, not from the coarse partition.** This is a critical detail of the Leiden paper: refinement does not start from the partition produced by local-move; it restarts each coarse community as singletons. This is what allows refinement to find substructure that local-move missed. The existing code has `let mut refined: Vec<usize> = (0..graph.n).collect();` which is correct — preserve that.
- **Refinement candidate set is bounded by coarse community.** A node `v` in coarse community `S` can only consider moving to sub-communities `C ⊂ S`. The `member_set.contains(&nbr)` filter in `try_merge_node` enforces this. Do not relax it.
- **Per-sub-community sigma_tot must include the 2× self-loop contribution.** When initialising `RefinementState::from_partition` for singleton init, `sigma_tot[i] = graph.degree[i]` (which already includes `2 × self_loops[i]` per M17). Don't add it again.
- **The argmax tie-break is by BTreeMap iteration order.** When multiple candidates have equal gain, the smallest-ID wins (BTreeMap iterates ascending). This is deterministic for a fixed seed but produces a slightly different partition than leidenalg's random-selection rule. Acceptable for v0; document in `docs/determinism.md` that the tie-break is deterministic-by-id, not random-proportional-to-gain.
- **`max_iter = 10` is fine.** Refinement converges in 2-3 iterations on well-structured graphs; 10 is generous. Don't reduce it without measuring.
- **The `gamma` parameter passed to refinement.** Currently `LeidenConfig::default().gamma = 1.0`. The simplified well-connectedness check uses this value. For modularity quality (`QualityFunction::Modularity`), the `gamma` parameter only affects the well-connectedness gate, not the gain formula. Keep `well_connected` cheap (O(1) check) so the gate doesn't dominate runtime.
- **Performance: avoid `BTreeSet::contains` in the inner loop.** The current `member_set: BTreeSet<usize>` lookup is O(log |S|) per neighbour. For large coarse communities (the `large` fixture has 100-node cliques), this adds up. Replace with a `Vec<bool>` of size `n` indexed by node ID for O(1) lookup. Build it once per `refine_community` call.
- **Performance: avoid `BTreeSet::collect` in the candidates set.** Same reason. Replace with a small `Vec<usize>` and `sort_unstable() + dedup()`. For low-degree nodes the candidate set is tiny; allocating a `BTreeSet` is wasteful.
- **Performance: avoid scanning all neighbours twice (once for candidates, once for k_in).** Combine the two loops: walk neighbours once, accumulate `k_in_per_comm: BTreeMap<usize, f64>`, then loop the map. Mirror `local_move_phase::best_neighbour_community`'s structure.
- **Modularity invariance under refinement.** Refinement should never *decrease* the modularity at the current level (since it only makes positive-gain moves). Add a property test or debug assertion. Note: this is at the *current level*, not after aggregation+recursion.
- **`refine_community` writes back into the shared `refined: &mut [usize]`.** Take care that simultaneous refinement of sibling coarse communities (which the loop does sequentially, not in parallel) doesn't accidentally cross-write. The `member_set` filter prevents reads outside the current coarse community; writes are scoped to `refined[member]` for `member in members`. Verify this is the case.
- **`well_connected` is informational for v0.** Even with `gamma = 1.0`, the simplified well-connectedness check may reject some moves that the exact Traag formulation would allow (or vice versa). For v0 we accept this — verify-leiden tolerance is 1% modularity, which is wide enough to absorb the difference. Document in the rustdoc on `well_connected` that it's a "v0 simplification — see Traag 2019 Algorithm 2 for the exact formulation."
- **Doc-comment placement when adding `RefinementState`.** Per CLAUDE.md "Doc comment placement when inserting items": ensure a blank line separates the new struct's `///` block from the next item's `///` block.
- **The `verify-leiden.yml` workflow's `paths:` filter.** If the user has touched `crates/sdivi-detection/**` since M17 closed, the workflow will fire on M18's push. Don't disable the path filter — it's saving CI minutes on PRs that don't touch detection.
- **Cargo lock churn.** This milestone shouldn't add or remove any dependency. If `Cargo.lock` changes outside the workspace's own crates, investigate before committing.
- **The `_probe.rs` scaffolding from the debugging session is gone.** Verify before commit that no `_probe.rs`-style ad-hoc test files exist under `crates/sdivi-detection/tests/`.

**Seeds Forward:**

- **Faithful Traag 2019 random selection** (probability proportional to `exp(ΔQ / θ)`) is a future polish. The current argmax + BTreeMap-tie-break is deterministic and simpler; it produces partition quality within the verify-leiden tolerance. A follow-up milestone can introduce the random selection rule and tune the temperature parameter `θ`. That change is **not** breaking (the modularity output stays within the same 1% tolerance band), but bit-identical reproducibility across the cutover would not hold; document this in `MIGRATION_NOTES.md` if/when it lands.
- **Exact γ-connectivity (Traag formulation)** can replace the v0 simplification in `well_connected`. The simplified check passes verify-leiden today; the exact check would reject more moves and potentially produce slightly higher-modularity partitions. Same non-breaking SemVer story as above.
- **Performance optimisations** (CSR view, parallel local-move within a coarse community, etc.) are deferred — modularity correctness is the gate for v0, not throughput. The KDD-5 "no CSR view" decision documented in M05's DRIFT_LOG remains in force; revisit only when a real perf bottleneck appears.
- **Re-baseline language for adopters.** Consumers who have stored snapshots from pre-M18 sdivi-rust will see "drift" between M16-era and M18-era snapshots that's purely from the algorithm correction. CHANGELOG and `MIGRATION_NOTES.md` must call this out explicitly so adopters don't mistake it for a real codebase-divergence signal.
- **Test coverage gap closed.** Going forward, every modularity-affecting change to `sdivi-detection` is gated by `verify-leiden.yml`. Don't merge a `crates/sdivi-detection/**` change with verify-leiden disabled or skipped.

---
