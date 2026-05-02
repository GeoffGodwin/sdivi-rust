# Drift Log

## Metadata
- Last audit: 2026-05-01
- Runs since audit: 3

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
- [2026-05-02 | "M18"] `mod.rs:138-147` — The pattern `if condition { break; }` immediately followed by `debug_assert!(!condition)` appears only once in the codebase. If this pattern is adopted elsewhere for invariant documentation, a convention note in `CLAUDE.md` would help future contributors distinguish "normal early-return" from "invariant-documenting dead assert."
- [2026-05-02 | "M18"] `refine.rs` — The `max_iter = 10` constant is a bare literal in `refine_community`. The local-move phase in `mod.rs` uses `cfg.max_iterations` passed down from `LeidenConfig`. Refinement's inner cap being a hardcoded literal (rather than a `LeidenConfig` field or named constant) is a mild inconsistency. No behaviour change needed for v0, but a `const MAX_REFINE_ITER: usize = 10;` at module scope would aid future tuning.
- [2026-05-02 | "M18"] `refinement.rs:295` — `prop_assert!` tolerance is `1e-9`; elsewhere in the test suite the convention is `1e-12`. Both are far tighter than any practical FMA drift, so this is purely cosmetic.
- [2026-05-02 | "M17"] `aggregate.rs:39` — `std::collections::BTreeMap` is imported via full path rather than a `use` statement at the top of the file. The rest of the codebase uses top-level `use` declarations. Cosmetic inconsistency, not a correctness issue.
- [2026-05-02 | "M17"] `modularity.rs:add_node` comment — "When `to == node` this is immediately overwritten by the self-loop addition below" accurately describes `inner_edges` but is silent about the sigma_tot/size double-increment on the same code path. If someone later reads this comment expecting the singleton round-trip to be fully no-op, they may be confused.
- [2026-05-01 | "Address all 9 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. Fix each item and note what you changed."] `bindings/sdivi-wasm/src/exports.rs:160-162` — `change_coupling: None` intentional gap is tracked only by a TODO comment inside the file. No corresponding ADL entry or issue exists to schedule the fix post-MVP. Risk of the TODO being silently forgotten.
- [2026-05-01 | "Address all 9 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. Fix each item and note what you changed."] `bindings/sdivi-wasm/src/types.rs:46-48` — `WasmLeidenConfigInput` missing `edge_weights` tracked as ADL-4. Verify ADL-4 actually exists in the architecture log; if not, create the entry so the gap is formally tracked.
- [2026-05-01 | "Address all 9 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. Fix each item and note what you changed."] `.tekhton/NON_BLOCKING_LOG.md` — all 9 items are marked `[x]` (resolved) but items 3, 6, and 7 were deferred rather than fixed. The log offers no way to distinguish "resolved by fixing" from "resolved by deferring," which will obscure the true open count in future audits. Consider a `[deferred]` marker for clarity.
- [2026-05-01 | "architect audit"] *(stays in DRIFT_LOG.md for next cycle)* None. All 9 (10 items counting stale sub-items) unresolved observations from the drift log are addressed above.

## Decisions (Declined / Will Not Implement)

## Resolved
