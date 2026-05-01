# Drift Log

## Metadata
- Last audit: 2026-04-30
- Runs since audit: 1

## Design Drift / Ratified
- [2026-04-29 | "consumer-app-driven scope shift"] **KDD-12 (sdi-core pure-compute reshape) and KDD-13 (WASM moves into v0) ratified.** Driver: a strict-mode TS consumer app at the user's workplace becomes the first concrete consumer of sdi-rust ahead of mid-June reviews. Today's `sdi-core` (Pipeline + I/O composition) cannot compile to WASM — transitively pulls `tree-sitter`, `walkdir`, `ignore`, `rayon`, `std::fs::*`. Plan: reshape the milestone schedule from M08 onward.
  - **New M08:** `sdi-core` Pure-Compute Reshape and WASM-readiness. Splits today's `sdi-core` into `sdi-core` (pure compute facade, WASM-compatible) and `sdi-pipeline` (orchestration crate owning FS/clock/atomic-write I/O). Adds `compute_*` functions over plain serde input structs (`DependencyGraphInput`, `PatternInstanceInput`, etc.), `normalize_and_hash` for foreign extractors, `compute_thresholds_check` for exit-10 logic. Feature-gates `sdi-graph`/`sdi-detection`/`sdi-patterns`/`sdi-snapshot` via `pipeline-records` (default ON for native, OFF for WASM).
  - **Old M08-M11 shift down:** former M08 (Trend/Check/Show CLI) → M09; former M09 (Boundaries) → M10; former M10 (Docs + bifl-tracker) → M11; former M11 (Release) → M13.
  - **New M12:** WASM Crate, npm Package, Consumer App Integration. `bindings/sdi-wasm` with `wasm-bindgen` + `tsify`-derived `.d.ts`, ships as `@geoffgodwin/sdi-wasm@0.1.0`.
  - **New M13:** Release pipeline now publishes both crates.io and npm behind the same manual approval gate.
  - **PyO3/napi-rs bindings** remain post-MVP / v1 era — no concrete consumer.
  - **Reason this is design drift:** DESIGN.md and the original CLAUDE.md said WASM was post-MVP per KD14 ("when a concrete consumer exists"). That condition now holds. The reshape must land before the 0.1.0 SemVer commitment in M13 (Rule 18). Reshaping `sdi-core` post-0.1.0 would force a 0.2.0 with breaking changes for the very first user (the consumer app).
  - **Files updated this cycle:** `CLAUDE.md` (KDD-12, KDD-13, Module Boundaries, Repository Layout, Critical System Rules 21–23, "What Not to Build Yet"); `.claude/milestones/MANIFEST.cfg`; `.claude/milestones/m08-sdi-core-pure-compute-reshape.md` (new); `.claude/milestones/m09-trend-check-show-remaining-cli-commands.md` (rewritten from old m08); `.claude/milestones/m10-boundaries-infer-ratify-show.md` (rewritten from old m09); `.claude/milestones/m11-documentation-examples-determinism-bifl-tracker.md` (rewritten from old m10); `.claude/milestones/m12-wasm-crate-and-consumer-app-integration.md` (new); `.claude/milestones/m13-release-pipeline-and-distribution.md` (rewritten from old m11). Old `m12-bindings-pyo3-and-napi-rs-post-mvp.md` retained as v1-era post-MVP placeholder (already excluded from manifest).

## Unresolved Observations
- [2026-04-30 | "Implement Milestone 9: Trend, Check, Show — Remaining CLI Commands"] `crates/sdi-cli/tests/version.rs:14` — Hardcoded version string is a systemic pattern that will break again on the next version bump. Low-risk but predictably recurring toil; replace with `env!("CARGO_PKG_VERSION")` when touching this file next.
- [2026-04-30 | "architect audit"] These entries remain in DRIFT_LOG.md for the indicated cycle:
- [2026-04-30 | "architect audit"] **`[2026-04-29 | "M08"] crates/sdi-core/tests/compute_thresholds_check.rs:96-104 — override_expiry_ignored_when_expired passes for wrong reason`** — Deferred to M09 start. The test currently passes because `cfg.overrides` is accepted but never read in M08's `compute_thresholds_check`. When M09 wires per-category overrides, a companion test must be added that uses an **active** (unexpired) override raising the limit above the test value and asserts the breach is suppressed — proving the override mechanism fires correctly, not just that defaults apply. Without that companion, the expiry test has no meaningful contrast case. Action owner: M09 coder, first task before wiring override logic.
- [2026-04-30 | "architect audit"] **`[2026-04-29 | "architect audit"] clap = ">=4.4, <4.5" version restriction`** — Deferred to M11 per prior decision. No action until M11 begins.

## Decisions (Declined / Will Not Implement)

## Resolved
- [RESOLVED 2026-04-30] `crates/sdi-cli/tests/version.rs:14` — Hardcoded version string is a systemic pattern that will break again on the next version bump. Low-risk but predictably recurring toil; replace with `env!("CARGO_PKG_VERSION")` when touching this file next.
- [RESOLVED 2026-04-30] `crates/sdi-core/tests/compute_thresholds_check.rs:96-104` — `override_expiry_ignored_when_expired` passes correctly, but because overrides are entirely unused in M08, not because expiry logic executed. A bug in M09's expiry path would not be caught by this test as written. Revisit when M09 adds override integration.
- [RESOLVED 2026-04-30] `crates/sdi-core/tests/compute_thresholds_check.rs:106-128` — two test functions whose bodies end in `let _ = r;` produce false-positive coverage statistics. Restructure when M09 adds behavioral assertions.
- [RESOLVED 2026-04-30] `.claude/milestones/MANIFEST.cfg` and the M08 milestone file could not be updated to `done` (permission denied, per coder note). Human operator should mark M08 done manually.
- [RESOLVED 2026-04-30] `leiden/mod.rs:173` — The singleton fallback `let target = if state.size[old_comm] == 0 { node } else { old_comm }` uses a raw node index (0..n) as a community ID. After the offset fix, real community IDs are n..n+k so a singleton at `node` < n is unambiguous — but this invariant (singleton ID == node index, always < n) is undocumented. A future caller that omits the offset guarantee reintroduces the collision silently.
- [RESOLVED 2026-04-30] `thresholds.rs:26` — `validate_date_format` accepts Feb 29 for any year (noted in fn doc comment). Intentional but inconsistent with strict calendar validation. No action required; logged for the audit accumulation.
- [RESOLVED 2026-04-30] Stays in DRIFT_LOG.md for next cycle (M11):
- [RESOLVED 2026-04-30] `[2026-04-29 | "architect audit"]` **`clap = ">=4.4, <4.5"` version restriction** — intentionally deferred to M11 per prior coder decision. No action until M11 begins.
