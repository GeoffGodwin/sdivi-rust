# Drift Log

## Metadata
- Last audit: 2026-05-02
- Runs since audit: 4

## Design Drift / Ratified
- [2026-04-29 | "consumer-app-driven scope shift"] **KDD-12 (sdivi-core pure-compute reshape) and KDD-13 (WASM moves into v0) ratified.** Driver: a strict-mode TS consumer app at the user's workplace becomes the first concrete consumer of sdivi-rust ahead of mid-June reviews. Today's `sdivi-core` (Pipeline + I/O composition) cannot compile to WASM — transitively pulls `tree-sitter`, `walkdir`, `ignore`, `rayon`, `std::fs::*`. Plan: reshape the milestone schedule from M08 onward.
  - **New M08:** `sdivi-core` Pure-Compute Reshape and WASM-readiness. Splits today's `sdivi-core` into `sdivi-core` (pure compute facade, WASM-compatible) and `sdivi-pipeline` (orchestration crate owning FS/clock/atomic-write I/O). Adds `compute_*` functions over plain serde input structs (`DependencyGraphInput`, `PatternInstanceInput`, etc.), `normalize_and_hash` for foreign extractors, `compute_thresholds_check` for exit-10 logic. Feature-gates `sdivi-graph`/`sdivi-detection`/`sdivi-patterns`/`sdivi-snapshot` via `pipeline-records` (default ON for native, OFF for WASM).
  - **Old M08-M11 shift down:** former M08 (Trend/Check/Show CLI) → M09; former M09 (Boundaries) → M10; former M10 (Docs + bifl-tracker) → M11; former M11 (Release) → M13.
  - **New M12:** WASM Crate, npm Package, Consumer App Integration. `bindings/sdivi-wasm` with `wasm-bindgen` + `tsify`-derived `.d.ts`, ships as `@geoffgodwin/sdivi-wasm@0.1.0`.
  - **New M13:** Release pipeline now publishes both crates.io and npm behind the same manual approval gate.
  - **PyO3/napi-rs bindings** remain post-MVP / v1 era — no concrete consumer.
  - **Reason this is design drift:** DESIGN.md and the original CLAUDE.md said WASM was post-MVP per KD14 ("when a concrete consumer exists"). That condition now holds. The reshape must land before the 0.1.0 SemVer commitment in M13 (Rule 18). Reshaping `sdivi-core` post-0.1.0 would force a 0.2.0 with breaking changes for the very first user (the consumer app).
  - **Files updated this cycle:** `CLAUDE.md` (KDD-12, KDD-13, Module Boundaries, Repository Layout, Critical System Rules 21–23, "What Not to Build Yet"); `.claude/milestones/MANIFEST.cfg`; `.claude/milestones/m08-sdivi-core-pure-compute-reshape.md` (new); `.claude/milestones/m09-trend-check-show-remaining-cli-commands.md` (rewritten from old m08); `.claude/milestones/m10-boundaries-infer-ratify-show.md` (rewritten from old m09); `.claude/milestones/m11-documentation-examples-determinism-bifl-tracker.md` (rewritten from old m10); `.claude/milestones/m12-wasm-crate-and-consumer-app-integration.md` (new); `.claude/milestones/m13-release-pipeline-and-distribution.md` (rewritten from old m11). Old `m12-bindings-pyo3-and-napi-rs-post-mvp.md` retained as v1-era post-MVP placeholder (already excluded from manifest).

## M12 / WASM Crate Changes (2026-05-01)
- [2026-05-01 | "M12"] **`tsify 0.4.x` is pre-1.0** — pinned in `[workspace.dependencies]`. Watch for breaking version bumps. The crate is widely used in the wasm-bindgen ecosystem but the maintainer does not guarantee SemVer until 1.0. If `0.5.x` introduces breaking changes the bump must be coordinated with the generated `.d.ts` output shape.
- [2026-05-01 | "M12"] **`wasm-bindgen 0.2.x` requires rustc ≥ 1.77** — This is within the project's 1.85.0 MSRV but breaks builds on older local toolchains. The local dev machine (1.75.0) cannot build `sdivi-wasm`; CI (1.85.0) is the canonical build path.
- [2026-05-01 | "M12"] **`sdivi-core` now re-exports inner-crate types** (`GraphMetrics`, `LeidenPartition`, `PatternCatalog`, `PatternStats`, `PatternFingerprint`). These were previously only reachable via internal crate paths. The re-exports are additive (backward-compatible) but widen the public surface of sdivi-core. Document in the "Module Boundaries" section of CLAUDE.md during M13 review.

## M18 Leiden Correctness Closure (2026-05-02)

- [2026-05-02 | "M17 + M18"] **M17 + M18 closed the Leiden correctness regression that was hidden
  by the absence of a modularity-asserting test outside `verify-leiden.yml`.** Pre-M17 partition
  tests verified `community_count() >= 1` and structural properties only, missing the modularity=0
  collapse caused by the fake sigma_tot in `refine.rs`. New regression gates: `prop_aggregate_modularity_invariance`
  (M17) and the `verify-leiden` suite (M18, now CI-blocking). Going forward, every
  `crates/sdivi-detection/**` change is gated by `verify-leiden.yml`; don't merge with verify-leiden
  disabled or skipped.

## Unresolved Observations
- [2026-05-02 | "Implement Milestone 21: Weighted Leiden on WASM"] `compute/mod.rs:9` — `threshold_types` is declared `mod threshold_types` (private) while all other compute submodules are `pub mod`. Types are accessible via `thresholds.rs`'s `pub use super::threshold_types::*`, so this is not a bug, but the asymmetry may surprise a future contributor. A brief comment on the `mod` line would forestall the question.
- [2026-05-02 | "Implement Milestone 20: Threshold-Comparison Epsilon for Cross-Arch Stability"] `compute/mod.rs:9` — `threshold_types` is declared `mod threshold_types` (private) while all other compute submodules are `pub mod`. Types are accessible via `thresholds.rs`'s `pub use super::threshold_types::*`, so this is not a bug, but the asymmetry may surprise a future contributor. A brief comment on the `mod` line would forestall the question.
- [2026-05-02 | "M19"] `crates/sdivi-pipeline/src/helpers.rs:55-70` — `graph_to_boundary_input` calls `dg.node_path(i)` in a `(0..n)` index range loop rather than iterating nodes directly. This pattern assumes `DependencyGraph` has a contiguous `0..node_count()` index space. If `DependencyGraph` ever uses non-contiguous indices (e.g. after node removal), this silently drops nodes. Consistent with current usage elsewhere but worth a `// SAFETY:` note on the assumption.
- [2026-05-02 | "M19"] `bindings/sdivi-wasm/src/exports.rs:165-184` — `assemble_snapshot` WASM wrapper passes `violation_count = 0` to the core function then manually overwrites `snap.intent_divergence` if `boundary_count` is `Some`. This is a second code path that assembles `IntentDivergenceInfo` outside of `sdivi_snapshot::assemble_snapshot`. Currently harmless but diverges from the single-assembly-seam principle over time.
- [2026-05-02 | "architect audit"] Stays in `NON_BLOCKING_LOG.md` for a future cycle.
- [2026-05-02 | "architect audit"] `quality.rs:compute_stability` — stability > 1.0 with self-loops (M17, NON_BLOCKING_LOG item 5). The code path is inert: `build_partition` always calls `compute_stability` on the original `LeidenGraph` constructed from a `DependencyGraph`, which has no self-loops (`self_loops[i] == 0.0` always). No v0 behaviour change needed. Revisit if `compute_stability` is ever exposed for aggregate-level introspection.
- [2026-05-02 | "architect audit"] `refine.rs:150` — `#[doc(hidden)]` on a function that also has a full `///` doc block (M18, NON_BLOCKING_LOG item 2). Confirmed correct: the pattern is consistent with `aggregate_network` and `LeidenGraph`; the hidden-doc + full-doc combination is the established codebase pattern for test-plumbing internal re-exports.
- [2026-05-02 | "architect audit"] `refine.rs:26` — `RefinementState` is `pub` rather than `pub(crate)` (M18, NON_BLOCKING_LOG item 3). Confirmed intentional: the `internal` module re-export requires `pub`; the pattern matches `LeidenGraph` and `AggregateResult`. No change.

## Decisions (Declined / Will Not Implement)

## Resolved
