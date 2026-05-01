# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [0.1.2] - 2026-05-01

### Added
- Implemented the full M15 change-coupling analyzer across all layers: (M15)

### Added
- Change-coupling analyzer wired up. New snapshot field `change_coupling`. New `boundaries.weighted_edges = true` mode multiplies import-edge weights by `(1.0 + frequency)`. New pure-compute entry point `sdi_core::compute_change_coupling` exported through WASM. Schema stays `1.0`.

## [0.1.1] - 2026-05-01

### Added
- Added `convention_drift_per_category: BTreeMap<String, f64>` to `PatternMetricsResult` (M14)

### Added
- **M14 — Per-category threshold override wiring**: `ThresholdsInput.overrides` and `ThresholdsInput.today` now actively filter per-category breaches against expiry. `PatternMetricsResult` gains `convention_drift_per_category: BTreeMap<String, f64>`. `DivergenceSummary` gains `pattern_entropy_per_category_delta` and `convention_drift_per_category_delta` (both `None` on the first-snapshot path). `ThresholdBreachInfo` gains `category: Option<String>` (absent for aggregate breaches). `ThresholdCheckResult` gains `applied_overrides: BTreeMap<String, AppliedOverrideInfo>` for diagnostic consumers. Snapshot schema stays `"1.0"` — all new fields are additive with `#[serde(default)]`.

## [0.1.0] - 2026-05-01

### Added
- **M01 — Workspace scaffold**: Cargo workspace with all 15 member crates, `[workspace.dependencies]`, resolver 2.
- **M02 — Config loading**: `Config::load_or_default` with 5-level precedence chain; `BoundarySpec` YAML reader; per-category threshold overrides with mandatory `expires`; `sdi init`.
- **M03 — Parsing stage**: `sdi-parsing` with `walkdir` + `ignore` + `rayon`; `LanguageAdapter` trait; CST-drop ownership invariant; Rust language adapter.
- **M04 — Language adapters**: Python, TypeScript, JavaScript, Go, Java adapters via tree-sitter grammars.
- **M05 — Dependency graph + native Leiden**: `sdi-graph` (`petgraph` dependency graph); native Rust Leiden port in `sdi-detection` (CPM + Modularity quality, no FFI); `LeidenPartition` with stability score; `sdi snapshot` baseline.
- **M06 — Pattern fingerprinting**: `sdi-patterns` with tree-sitter queries, `blake3`-keyed fingerprints, `PatternCatalog` with entropy; `normalize_and_hash` canonical entry point.
- **M07 — Snapshot assembly, delta, persistence**: `assemble_snapshot`, `compute_delta` (null on first snapshot), atomic write + retention enforcement in `sdi-pipeline::store`.
- **M08 — `sdi-core` pure-compute reshape**: `sdi-core` reshaped to WASM-compatible pure-compute facade; `sdi-pipeline` extracted as orchestration crate; `compute_*` functions over `*Input` serde structs; `pipeline-records` feature gates on inner crates.
- **M09 — CLI commands**: `sdi trend`, `sdi check` (exit 10 on threshold breach), `sdi show` (text + JSON); stdout/stderr discipline; exit-code test suite.
- **M10 — Boundaries**: `sdi boundaries infer`, `sdi boundaries ratify`, `sdi boundaries show`; `infer_boundaries` and `compute_boundary_violations` in `sdi-core`.
- **M11 — Documentation, examples, determinism polish**: `docs/` directory (cli-integration, library-embedding, migrating-from-sdi-py, determinism); `examples/` (embed_pipeline.rs, embed_compute.rs, custom_config.rs, binding_node.ts); bifl-tracker validation harness; `normalize_and_hash` cross-platform determinism tests.
- **M12 — WASM crate + npm package**: `bindings/sdi-wasm` crate with wasm-bindgen + tsify-derived `.d.ts`; all `sdi_core::compute_*` functions exported; `normalize_and_hash` exported for foreign extractors; `@geoffgodwin/sdi-wasm` npm package; WASM CI workflow with cross-platform hash determinism check.
- **M13 — Release pipeline**: Tag-driven `.github/workflows/release.yml`; matrix binary builds for Linux x86_64+aarch64, macOS x86_64+aarch64, Windows x86_64; stripped LTO binaries attached to GitHub Release; manual-approval-gated crates.io publish (11 crates in dependency order); manual-approval-gated npm publish; `cargo audit` weekly cron.

### Changed
- Workspace version: `0.0.16` → `0.1.0` (SemVer commitment baseline; `pub` items in `sdi-core` are now stable API).
- `[profile.release]` adds `lto = "thin"`, `strip = true`, `panic = "abort"` for smaller native binaries.

### Binary and bundle sizes (measured at first release build)
- `sdi-x86_64-unknown-linux-gnu`: ~TBD MiB
- `sdi-aarch64-unknown-linux-gnu`: ~TBD MiB
- `sdi-x86_64-apple-darwin`: ~TBD MiB
- `sdi-aarch64-apple-darwin`: ~TBD MiB
- `sdi-x86_64-pc-windows-msvc.exe`: ~TBD MiB
- `sdi_wasm_bg.wasm` (bundler target): ~TBD KiB

- [MILESTONE 12 ✓] feat: M12 (M13)
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
