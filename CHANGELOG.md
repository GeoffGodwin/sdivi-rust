# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.7] - 2026-05-02

### Fixed

- **Leiden refinement phase rewritten to use real per-sub-community Σ_tot** (M18). The previous
  `best_candidate` in `refine.rs` used a count of the node's neighbours in a sub-community as a
  fake sigma_tot, causing the gain formula to be almost always positive and producing a near-random
  refined partition. With M17's aggregate fixes in place, this caused all nodes to collapse into
  one community (modularity ≈ 0.0). The rewrite introduces `RefinementState` (tracking real
  `sigma_tot`, `inner_edges`, `size`) and `well_connected` (γ-connectivity gate, v0 simplification
  of Traag 2019 §2.2). All three verify-leiden fixtures (small/medium/large) now pass within
  1 % modularity tolerance against `leidenalg`. The verify-leiden CI workflow is re-enabled on
  push/PR with a 30-min job-level timeout safeguard.

### Note for adopters

  Snapshot `modularity` values for snapshots taken with pre-M18 sdivi-rust will differ from M18+
  snapshots because the previous algorithm produced incorrect (collapsed) partitions. Deltas
  spanning the M16→M18 boundary may show artificial "drift" that is purely an algorithm-correction
  artefact. Recommendation: re-baseline at the M18 boundary or compare only M18-era snapshots.

- Rewrote `refine.rs` with `RefinementState` struct, `apply_move`, `move_gain`, `well_connected`, (M18)
## [0.1.6] - 2026-05-02

### Added
- **`LeidenGraph` self-loops support** (`graph.rs`): Added `self_loops: Vec<f64>` field. (M17)

### Added

- Claude Code knowledge skill at `.claude/skills/sdivi/` — a router-style
  `SKILL.md` plus task-keyed sub-files (`cli.md`, `config.md`, `embedding.md`,
  `invariants.md`) so contributors and embedders using Claude get surgical SDIVI
  knowledge on demand instead of preloading `CLAUDE.md`.

### Fixed

- Leiden algorithm: `LeidenGraph` now supports self-loops (`self_loops` field).
  `aggregate_network` correctly preserves intra-community weight as self-loops on
  the aggregate super-nodes and no longer double-counts inter-community edges (the
  prior code visited each undirected edge twice, once per endpoint).
  `compute_modularity` and `compute_stability` now count self-loop weight as
  internal community weight, as required by Traag et al. 2019.  Per-step
  correctness verified by new aggregate-invariance unit tests; full algorithm
  correctness (verify-leiden fixture gate) is gated by M18.

## [0.1.5] - 2026-05-01

### Added
- Address all 10 open non-blocking notes in .tekhton/NON_BLOCKING_LOG.md.

## [0.1.4] - 2026-05-01

### Added
- All 10 non-blocking tech debt items from `.tekhton/NON_BLOCKING_LOG.md` addressed:
## [0.1.3] - 2026-05-01

### Added
- `crates/sdivi-pipeline/src/commit_extract.rs` (NEW) — ref resolution, UTC date normalization, tree extraction via git archive | tar (M16)

### Changed
- **M16 — `sdivi snapshot --commit REF` now analyzes the actual tree at REF.**
  Previously the flag only populated the snapshot's `commit` label while still
  parsing the working directory, silently producing snapshots with mismatched
  content. Now the pipeline resolves `REF` to a full 40-char SHA, extracts the
  tree via `git archive | tar`, runs all five stages against it, and writes a
  snapshot labeled with the resolved SHA and the commit's **commit-date**
  (normalised to UTC). The supplied `timestamp` argument is **overridden** when
  `--commit` is set, so trend ordering tracks chronology rather than
  wall-clock invocation time. Change-coupling history is collected ending at
  the resolved SHA (not HEAD). Pre-v0 callers relying on the prior label-only
  behavior must adjust.
- **`CommitExtractError`** added to `sdivi_pipeline::PipelineError` as
  `PipelineError::CommitExtract`. Propagates structured git diagnostic output
  (`stderr`) to callers. `--commit nonexistent-ref` exits with code 1.

## [0.1.2] - 2026-05-01

### Added
- **M15 — Change-coupling analyzer**: New snapshot field `change_coupling`. New `boundaries.weighted_edges = true` mode multiplies import-edge weights by `(1.0 + frequency)`. New pure-compute entry point `sdivi_core::compute_change_coupling` exported through WASM. Schema stays `"1.0"`.

## [0.1.1] - 2026-05-01

### Added
- **M14 — Per-category threshold override wiring**: `ThresholdsInput.overrides` and `ThresholdsInput.today` now actively filter per-category breaches against expiry. `PatternMetricsResult` gains `convention_drift_per_category: BTreeMap<String, f64>`. `DivergenceSummary` gains `pattern_entropy_per_category_delta` and `convention_drift_per_category_delta` (both `None` on the first-snapshot path). `ThresholdBreachInfo` gains `category: Option<String>` (absent for aggregate breaches). `ThresholdCheckResult` gains `applied_overrides: BTreeMap<String, AppliedOverrideInfo>` for diagnostic consumers. Snapshot schema stays `"1.0"` — all new fields are additive with `#[serde(default)]`.

## [0.1.0] - 2026-05-01

### Added
- **M01 — Workspace scaffold**: Cargo workspace with all 15 member crates, `[workspace.dependencies]`, resolver 2.
- **M02 — Config loading**: `Config::load_or_default` with 5-level precedence chain; `BoundarySpec` YAML reader; per-category threshold overrides with mandatory `expires`; `sdivi init`.
- **M03 — Parsing stage**: `sdivi-parsing` with `walkdir` + `ignore` + `rayon`; `LanguageAdapter` trait; CST-drop ownership invariant; Rust language adapter.
- **M04 — Language adapters**: Python, TypeScript, JavaScript, Go, Java adapters via tree-sitter grammars.
- **M05 — Dependency graph + native Leiden**: `sdivi-graph` (`petgraph` dependency graph); native Rust Leiden port in `sdivi-detection` (CPM + Modularity quality, no FFI); `LeidenPartition` with stability score; `sdivi snapshot` baseline.
- **M06 — Pattern fingerprinting**: `sdivi-patterns` with tree-sitter queries, `blake3`-keyed fingerprints, `PatternCatalog` with entropy; `normalize_and_hash` canonical entry point.
- **M07 — Snapshot assembly, delta, persistence**: `assemble_snapshot`, `compute_delta` (null on first snapshot), atomic write + retention enforcement in `sdivi-pipeline::store`.
- **M08 — `sdivi-core` pure-compute reshape**: `sdivi-core` reshaped to WASM-compatible pure-compute facade; `sdivi-pipeline` extracted as orchestration crate; `compute_*` functions over `*Input` serde structs; `pipeline-records` feature gates on inner crates.
- **M09 — CLI commands**: `sdivi trend`, `sdivi check` (exit 10 on threshold breach), `sdivi show` (text + JSON); stdout/stderr discipline; exit-code test suite.
- **M10 — Boundaries**: `sdivi boundaries infer`, `sdivi boundaries ratify`, `sdivi boundaries show`; `infer_boundaries` and `compute_boundary_violations` in `sdivi-core`.
- **M11 — Documentation, examples, determinism polish**: `docs/` directory (cli-integration, library-embedding, migrating-from-sdi-py, determinism); `examples/` (embed_pipeline.rs, embed_compute.rs, custom_config.rs, binding_node.ts); `normalize_and_hash` cross-platform determinism tests.
- **M12 — WASM crate + npm package**: `bindings/sdivi-wasm` crate with wasm-bindgen + tsify-derived `.d.ts`; all `sdivi_core::compute_*` functions exported; `normalize_and_hash` exported for foreign extractors; `@geoffgodwin/sdivi-wasm` npm package; WASM CI workflow with cross-platform hash determinism check.
- **M13 — Release pipeline**: Tag-driven `.github/workflows/release.yml`; matrix binary builds for Linux x86_64+aarch64, macOS x86_64+aarch64, Windows x86_64; stripped LTO binaries attached to GitHub Release; manual-approval-gated crates.io publish (11 crates in dependency order); manual-approval-gated npm publish; `cargo audit` weekly cron.

### Changed
- Workspace version: `0.0.16` → `0.1.0` (SemVer commitment baseline; `pub` items in `sdivi-core` are now stable API).
- `[profile.release]` adds `lto = "thin"`, `strip = true`, `panic = "abort"` for smaller native binaries.

### Binary and bundle sizes (measured at first release build)
- `sdivi-x86_64-unknown-linux-gnu`: ~TBD MiB
- `sdivi-aarch64-unknown-linux-gnu`: ~TBD MiB
- `sdivi-x86_64-apple-darwin`: ~TBD MiB
- `sdivi-aarch64-apple-darwin`: ~TBD MiB
- `sdivi-x86_64-pc-windows-msvc.exe`: ~TBD MiB
- `sdivi_wasm_bg.wasm` (bundler target): ~TBD KiB

[0.1.0] is the SemVer commitment baseline. Pre-release internal milestone
iteration (M01–M13, 2026-04-28 → 2026-05-01) was not published to crates.io
or tagged in git; per-iteration entries that previously appeared below this
line have been folded into the [0.1.0] block above. See git history for the
development trail.
