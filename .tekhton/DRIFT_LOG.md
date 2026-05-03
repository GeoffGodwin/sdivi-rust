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
- [2026-05-03 | "Address all 19 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. Fix each item and note what you changed."] `bindings/sdivi-wasm/src/weight_keys.rs:97` — `rejects_nan_weight` test asserts `e.contains("NaN")`, which passes because `format!("{}", f64::NAN)` == `"NaN"`. Works today but is an implementation-detail assertion. Low-risk, no action required.
- [2026-05-03 | "Address all 19 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. Fix each item and note what you changed."] `.tekhton/DRIFT_LOG.md:36` (carried from M23) — `CATEGORIES` and `CATEGORY_DESCRIPTIONS` parallel arrays in `sdivi-core/src/categories.rs` have no compile-time sync enforcement; runtime tests are the only guard. Not new; already noted in the drift log.
- [2026-05-03 | "Implement Milestone 24: Node.js WASM Distribution Target"] `tests/node_smoke/package.json` `"test"` script uses `node --input-type=module < index.mjs` (stdin redirect) while the CI step uses `node index.mjs` directly. Both work, but running `npm test` locally exercises a different invocation path than CI. Align to `node index.mjs` for consistency.
- [2026-05-03 | "Implement Milestone 24: Node.js WASM Distribution Target"] `bindings/sdivi-wasm/package.json` (the old single-target manifest at the binding root) is superseded by `pkg-template/package.json` but was intentionally left in place (noted in CODER_SUMMARY Observed Issues). It will cause confusion for contributors. A follow-up cleanup PR should delete or annotate it.
- [2026-05-03 | "Implement Milestone 24: Node.js WASM Distribution Target"] Prior cycle observations not addressed (out of scope for M24, carry forward): `WasmCategoryInfo`/`WasmCategoryCatalog` missing `PartialEq`; `list_categories()` placement in `exports.rs`; `CATEGORIES`/`CATEGORY_DESCRIPTIONS` parallel arrays.
- [2026-05-03 | "Implement Milestone 23: Pattern Category Contract + WASM `list_categories()`"] `crates/sdivi-core/src/categories.rs:24,35` — `CATEGORIES` and `CATEGORY_DESCRIPTIONS` are two parallel arrays that must stay in sync (same names, same order) with no compile-time enforcement. The runtime tests catch drift. A single combined source-of-truth array (e.g. `const CATALOG_ENTRIES: &[(&str, &str)]`) iterated by both `list_categories()` and `CATEGORIES` would eliminate the possibility of the two diverging silently between tests runs.

## Decisions (Declined / Will Not Implement)

## Resolved
- [2026-05-02 | "M20 run"] `crates/sdivi-detection/src/leiden/compute/mod.rs:9` — stale `use` of a removed helper. Fix applied in M20 cycle; imports corrected.
- [2026-05-02 | "M21 run"] `crates/sdivi-detection/src/leiden/compute/mod.rs:9` — duplicate observation carried from M20; same fix confirmed present. No further action needed.
- [2026-05-02 | "M19 run"] `crates/sdivi-detection/src/leiden/helpers.rs:55-70` — `build_leiden_graph` accepted a mutable reference where a shared reference sufficed. Fix applied; signature corrected in M19 cycle.
- [2026-05-02 | "architect audit"] `crates/sdivi-detection/src/leiden/quality.rs:compute_stability` — plan review confirmed function is inert (called only in tests, no pipeline impact). No code change required; observation closed.
- [2026-05-02 | "architect audit"] `crates/sdivi-detection/src/leiden/refine.rs:150` `#[doc(hidden)]` — attribute is intentional; `RefinementStep` is a public-but-not-API enum variant used by the verify-leiden test suite. No change required; observation closed.
- [2026-05-02 | "architect audit"] `crates/sdivi-detection/src/leiden/refine.rs:26` `RefinementState` pub — `pub` visibility on `RefinementState` is intentional for the same verify-leiden test access pattern. No change required; observation closed.
