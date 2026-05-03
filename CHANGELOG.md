# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.10] - 2026-05-02

### Changed

- Threshold gate now applies a `1e-9` epsilon to the limit in `compute_thresholds_check`,
  eliminating cross-arch gate flap from documented per-arch ULP drift in delta computations.
  Behaviour for any user-meaningful threshold is unchanged. (M20)
- `pub const THRESHOLD_EPSILON: f64 = 1e-9` added to `sdivi-core` and re-exported
  from `sdivi_core`. WASM and other embedders can reference the same constant.

- Added `pub const THRESHOLD_EPSILON: f64 = 1e-9` to `crates/sdivi-core/src/compute/threshold_types.rs` (new file, extracted from `thresholds.rs`) with a doc comment citing `docs/determinism.md § Threshold gate stability`. (M20)
## [0.1.9] - 2026-05-02

### Added
- Implemented real `compute_boundary_violations` in `crates/sdivi-core/src/compute/boundaries.rs`: (M19)

### Fixed

- `compute_boundary_violations` now performs real violation detection instead of
  always returning zero. Factor 4 (boundary violation velocity) is now active in
  `sdivi check`. Adopters with a `.sdivi/boundaries.yaml` should expect the first
  post-M19 snapshot to surface existing violations as a delta against the prior
  always-zero baseline. See `docs/cli-integration.md` for the recommended
  one-time `boundary_violation_rate` override to absorb the cutover.

### Changed

- `assemble_snapshot` now accepts a `violation_count: u32` parameter (M19).
  All existing callers must pass the computed count (or `0` if no boundary spec
  is present). This is a semver-minor change on `sdivi-snapshot`.

## [0.1.8] - 2026-05-02

### Changed

- Cleared the open non-blocking note backlog (six items in
  `.tekhton/NON_BLOCKING_LOG.md`). Most were doc tightening, error message
  polish, and small test additions. No public API changed; snapshot schema
  remains `"1.0"`.

### Fixed

- `bindings/sdivi-wasm/package.json` and `Cargo.lock` are now kept in sync with
  the workspace version. A mismatched version had previously slipped through
  CI.

## [0.1.7] - 2026-05-02

### Fixed

- Leiden refinement phase now uses real per-sub-community Σ_tot. The previous
  `best_candidate` in `refine.rs` substituted "neighbours in a sub-community"
  for `sigma_tot`, so the move-gain formula was almost always positive and the
  refined partition was effectively random. Combined with the M17 aggregate
  fixes, this caused all nodes to collapse into one community and modularity
  fell to roughly zero. The rewrite introduces `RefinementState` (tracking
  `sigma_tot`, `inner_edges`, `size`) and a `well_connected` gate (a v0
  simplification of the γ-connectivity rule from Traag 2019 §2.2). All three
  `verify-leiden` fixtures (small, medium, large) now pass within the 1 %
  modularity tolerance against `leidenalg`. The `verify-leiden` CI workflow
  is re-enabled on push and PR with a 30-min job-level timeout.

### Note for adopters

Snapshots produced before this release have a `modularity` value derived from
the broken refinement phase. Deltas that span the M16 → M18 boundary will
show artificial drift that is purely an algorithm-correction artefact.
Re-baseline at this release if you want comparable trend data.

## [0.1.6] - 2026-05-02

### Added

- `LeidenGraph.self_loops: Vec<f64>` so the aggregate network can preserve
  intra-community weight as self-loops on super-nodes (M17).
- Claude Code knowledge skill at `.claude/skills/sdivi/` (router-style
  `SKILL.md` plus `cli.md`, `config.md`, `embedding.md`, `invariants.md`).
  Contributors and embedders using Claude get task-keyed SDIVI knowledge on
  demand without preloading `CLAUDE.md`.

### Fixed

- `aggregate_network` no longer double-counts inter-community edges. The
  previous code visited each undirected edge twice (once per endpoint).
- `compute_modularity` and `compute_stability` count self-loop weight as
  internal community weight, as required by Traag et al. 2019. Per-step
  correctness is verified by new aggregate-invariance unit tests; full
  algorithm correctness is verified by the `verify-leiden` fixture gate
  reinstated in 0.1.7.

## [0.1.5] - 2026-05-01

### Changed

- Cleared a 10-item batch of non-blocking notes. Touched docs, error messages,
  and unit-test coverage; no public API or schema change.

## [0.1.4] - 2026-05-01

### Changed

- Cleared a 10-item batch of non-blocking notes. Touched docs, error messages,
  and unit-test coverage; no public API or schema change.

## [0.1.3] - 2026-05-01

### Changed

- `sdivi snapshot --commit REF` now analyses the actual tree at `REF` (M16).
  Previously the flag only stamped the snapshot's `commit` label while still
  parsing the working directory, silently producing snapshots with mismatched
  content. The pipeline now resolves `REF` to a full 40-char SHA, extracts the
  tree via `git archive | tar`, runs all five stages against it, and writes a
  snapshot labelled with the resolved SHA and the commit's commit-date
  (UTC-normalised). The supplied `timestamp` argument is overridden when
  `--commit` is set so trend ordering tracks chronology rather than wall-clock
  invocation order. Change-coupling history is collected ending at the
  resolved SHA.

### Added

- `crates/sdivi-pipeline/src/commit_extract.rs` (new module): ref resolution,
  UTC date normalisation, tree extraction.
- `PipelineError::CommitExtract` carries structured git diagnostic output
  (`stderr`) to callers. `--commit nonexistent-ref` exits 1.

## [0.1.2] - 2026-05-01

### Added

- Change-coupling analyser (M15). New snapshot field `change_coupling`.
  `boundaries.weighted_edges = true` multiplies import-edge weights by
  `(1.0 + frequency)`. Pure-compute entry point
  `sdivi_core::compute_change_coupling` is exported through WASM. Schema stays
  `"1.0"`.

## [0.1.1] - 2026-05-01

### Added

- Per-category threshold override wiring (M14). `ThresholdsInput.overrides`
  and `ThresholdsInput.today` actively filter per-category breaches against
  expiry. `PatternMetricsResult` gains
  `convention_drift_per_category: BTreeMap<String, f64>`. `DivergenceSummary`
  gains `pattern_entropy_per_category_delta` and
  `convention_drift_per_category_delta` (both `None` on the first-snapshot
  path). `ThresholdBreachInfo` gains `category: Option<String>` (absent for
  aggregate breaches). `ThresholdCheckResult` gains
  `applied_overrides: BTreeMap<String, AppliedOverrideInfo>` for diagnostic
  consumers. Snapshot schema stays `"1.0"`; all new fields are additive with
  `#[serde(default)]`.

## [0.1.0] - 2026-05-01

First public release of sdivi-rust. SemVer commitment baseline: `pub` items in
`sdivi-core` are stable API and any breaking change requires a major version
bump.

### Added

- Cargo workspace with 17 member crates, `[workspace.dependencies]`,
  resolver 2 (M01).
- `Config::load_or_default` with a 5-level precedence chain. `BoundarySpec`
  YAML reader. Per-category threshold overrides with mandatory `expires`.
  `sdivi init`. (M02)
- `sdivi-parsing` with `walkdir` + `ignore` + `rayon`. `LanguageAdapter`
  trait. CST-drop ownership invariant. (M03)
- Language adapters for Rust, Python, TypeScript, JavaScript, Go, and Java
  via tree-sitter grammars. (M04)
- `sdivi-graph` (`petgraph` dependency graph). Native Rust Leiden port in
  `sdivi-detection` (CPM and Modularity quality, no FFI). `LeidenPartition`
  with stability score. `sdivi snapshot` baseline. (M05)
- `sdivi-patterns` with tree-sitter queries, `blake3`-keyed fingerprints,
  `PatternCatalog` with entropy. `normalize_and_hash` canonical entry point.
  (M06)
- `assemble_snapshot`, `compute_delta` (`null` on first snapshot), atomic
  write and retention enforcement in `sdivi-pipeline::store`. (M07)
- `sdivi-core` reshaped as a WASM-compatible pure-compute facade.
  `sdivi-pipeline` extracted as the orchestration crate. `compute_*`
  functions over `*Input` serde structs. The `pipeline-records` feature gates
  the FS-touching paths on inner crates. (M08)
- `sdivi trend`, `sdivi check` (exit 10 on threshold breach), `sdivi show`
  (text and JSON). Stdout/stderr discipline. Exit-code test suite. (M09)
- `sdivi boundaries infer`, `sdivi boundaries ratify`, `sdivi boundaries show`.
  `infer_boundaries` and `compute_boundary_violations` in `sdivi-core`. (M10)
- `docs/` (`cli-integration`, `library-embedding`, `migrating-from-sdi-py`,
  `determinism`). `examples/` (`embed_pipeline.rs`, `embed_compute.rs`,
  `custom_config.rs`, `binding_node.ts`).
  `normalize_and_hash` cross-platform determinism tests. (M11)
- `bindings/sdivi-wasm` with `wasm-bindgen` + `tsify`-derived `.d.ts`. All
  `sdivi_core::compute_*` functions exported. `normalize_and_hash` exported
  for foreign extractors. `@geoffgodwin/sdivi-wasm` npm package. WASM CI
  workflow with cross-platform hash determinism check. (M12)
- Tag-driven `.github/workflows/release.yml`. Matrix binary builds for
  Linux x86_64 + aarch64, macOS x86_64 + aarch64, Windows x86_64. Stripped
  LTO binaries attached to each GitHub Release. Manual-approval-gated
  crates.io publish (11 crates in dependency order). Manual-approval-gated
  npm publish. `cargo audit` weekly cron. (M13)

### Changed

- Workspace version `0.0.16` → `0.1.0` (SemVer commitment baseline).
- `[profile.release]` adds `lto = "thin"`, `strip = true`, `panic = "abort"`
  for smaller native binaries.
