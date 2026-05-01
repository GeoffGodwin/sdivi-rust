# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.16] - 2026-05-01

### Added
- Managed the human_action_required and trimmed the CLAUDE.md a little. (M12)
## [0.0.15] - 2026-05-01

### Added
- [MILESTONE 10 ✓] feat: Implement Milestone 10: Boundaries — Infer, Ratify, Show (M11)

## [0.0.14] - 2026-04-30

### Added
- [MILESTONE 9 ✓] feat: Implement Milestone 9: Trend, Check, Show — Remaining CLI Commands (M10)
## [0.0.13] - 2026-04-30

### Added
- [MILESTONE 08 ✓] feat: M08 (M9)

## [0.0.12] - 2026-04-30

### Added
- M08 (`sdi-core` Pure-Compute Reshape) was already implemented in a prior session (milestone status: "done"). The workspace had one stale failing test from that implementation: (M08)
## [0.0.11] - 2026-04-30

### Added
- **Note 1 — `compute_thresholds_check` overrides/today not yet wired (M09):**

## [0.0.10] - 2026-04-29

### Added
- [MILESTONE 08 ✓] feat: M08 (M8)
## [0.0.9] - 2026-04-29

### Added (M08 — sdi-core Pure-Compute Reshape)
- New crate `sdi-pipeline`: orchestration entry point owning `Pipeline::snapshot`, warm-start cache I/O, atomic snapshot writes, and retention enforcement.
- `sdi-core` reshaped to pure-compute WASM-compatible facade. Exposes `compute_*` functions and `*Input` serde struct family (`DependencyGraphInput`, `PatternInstanceInput`, `LeidenConfigInput`, `ThresholdsInput`, `BoundarySpecInput`, `PriorPartition`, `NormalizeNode`, `validate_node_id`).
- `sdi-core::compute`: `compute_coupling_topology`, `detect_boundaries`, `compute_boundary_violations`, `compute_pattern_metrics`, `compute_thresholds_check`, `normalize_and_hash`. All pure; all callable from WASM.
- `sdi-snapshot`: new modules `trend.rs` (`compute_trend`, `TrendResult`) and `boundary_inference.rs` (`infer_boundaries`, `BoundaryInferenceResult`).
- Cargo feature `pipeline-records` added to `sdi-graph`, `sdi-detection`, `sdi-patterns`, `sdi-snapshot` (default ON; OFF for WASM builds via sdi-core).
- Cargo feature `loader` added to `sdi-config` (default ON; gates all FS/clock code).

### Changed (M08)
- **Breaking rename** (pre-1.0): `build_snapshot` → `assemble_snapshot`. Gains `pattern_metrics: PatternMetricsResult` argument.
- `Snapshot` gains `pattern_metrics: PatternMetricsResult` field (`convention_drift: f64`, `entropy_per_category`).
- `DivergenceSummary` gains `convention_drift_delta: Option<f64>`; `compute_delta` now populates it.
- `normalize_and_hash(kind, children)` is the canonical fingerprint entry point; `fingerprint_node_kind(kind)` is now a thin wrapper — M07 catalog output is byte-identical.
- Override expiry moved from config-load-time to `compute_thresholds_check` via caller-supplied `ThresholdsInput::today: NaiveDate` — no more `SystemTime::now()` in `sdi-config`.
- **sdi-core public-API change:** `ThresholdsInput::default().today` now uses far-future sentinel `9999-12-31` (was `2026-01-01`). This ensures all per-category threshold overrides are treated as expired by default — a conservative failure mode. Embedders using per-category overrides must supply the real current date explicitly.
- `sdi-detection::warm_start` FS ops moved to `sdi-pipeline::cache`; pure mapping logic remains.
- `sdi-snapshot::store` FS ops moved to `sdi-pipeline::store`.

## [0.0.8] - 2026-04-29

### Added
- [MILESTONE 06 ✓] feat: M06 (M7)
## [0.0.7] - 2026-04-29

### Added
- Responded to the HUMAN_ACTION_REQUIRED and removed an item from the drift log. (M06)

## [0.0.6] - 2026-04-29

### Added
- [MILESTONE 04 ✓] feat: M04 (M5)
## [0.0.5] - 2026-04-29

### Added
- Address all 7 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md. F (M04)

## [0.0.4] - 2026-04-29

### Added
- Added `crates/sdi-lang-rust/build.rs` as a layout-conformance placeholder (item 1)
## [0.0.3] - 2026-04-29

### Added
- [MILESTONE 2 ✓] feat: Implement Milestone 2: Config Loading + Boundary Spec Reader (M3)

## [0.0.2] - 2026-04-28

### Added
- [MILESTONE 01 ✓] feat: M01 (M2)
## [0.0.1] - 2026-04-28

### Added
- **Cargo workspace** (`Cargo.toml`): all 15 member crates wired up; `[workspace.dependencies]` lists every external dep with pinned ranges; `resolver = "2"`. (M01)
