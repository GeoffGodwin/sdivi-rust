# Coder Summary
## Status: COMPLETE

## What Was Implemented
### Milestone 11: Documentation, Examples, Determinism Polish, bifl-tracker Validation

**Bug Fixes (from Tester Report):**
- `PipelineError::NoGrammarsAvailable` now emitted: after `parse_repository` returns an empty
  `Vec`, the pipeline calls `collect_files` and checks if any extensioned files exist outside
  `.sdi/`. If so, exits with `PipelineError::NoGrammarsAvailable` → exit code 3.
- `error_exit_code` in `main.rs` now downcasts `PipelineError::NoGrammarsAvailable` →
  `ExitCode::AnalysisError` (3).
- Test `snapshot_exits_zero_on_all_unknown_languages` renamed
  `snapshot_exits_three_on_all_unknown_languages` and updated to expect `.code(3)`.

**Documentation:**
- `README.md` expanded: quick start, install paths (cargo, binary, npm), overview,
  embedding section, docs links. Under 200 lines.
- `docs/cli-integration.md` (NEW): GHA snippet, exit-code reference table, threshold config,
  command reference, env vars, retention docs.
- `docs/library-embedding.md` (NEW): Two sections — orchestration path (`sdi-pipeline`) and
  pure-compute/WASM path (`sdi-core`). Includes consumer-app pattern, `normalize_and_hash`
  usage, NodeId canonicalization, WASM TypeScript example.
- `docs/migrating-from-sdi-py.md` expanded: full carry/change matrix (snapshot schema break,
  new exit code 3, fingerprint change, boundary inference, YAML comment loss). Replaces stub.
- `docs/determinism.md` (NEW): BTreeMap discipline, seed contract, pattern fingerprints,
  `normalize_and_hash` for foreign extractors, FMA cross-platform notes, pure-function table,
  property tests reference.
- Fixed pre-existing `cargo doc` warnings: `[Snapshot]` → `[crate::Snapshot]` in facade.rs,
  `[FeatureRecord]` → backtick in dependency_graph.rs, `[ACTIVE_TREES]` → backtick in lib.rs.

**Examples (in `crates/sdi-cli/examples/`):**
- `embed_pipeline.rs`: Full FS pipeline via `sdi_pipeline::Pipeline`; defaults to
  `tests/fixtures/simple-rust`. `cargo run --example embed_pipeline` succeeds.
- `embed_compute.rs`: Pure-compute path via `sdi_core::compute_*`; parity check asserts
  node count matches pipeline path. `cargo run --example embed_compute` succeeds.
- `custom_config.rs`: Programmatic `Config` construction with fixed seed and custom
  thresholds. `cargo run --example custom_config` succeeds.

**Proptest Additions:**
- `crates/sdi-core/tests/prop_thresholds.rs` (NEW): `prop_test_compute_thresholds_check_pure`
  — verifies referential transparency and null-summary never-breach invariant (500 cases).
- `crates/sdi-patterns/tests/prop_normalize.rs` (NEW): `prop_test_normalize_and_hash_stable`
  — verifies same AST input → same blake3 digest, result is 64-char hex, different kinds
  produce different digests (500 cases).
- `crates/sdi-pipeline/tests/prop_pipeline.rs` (NEW): `prop_test_pipeline_deterministic`
  — verifies same seed → bit-identical Snapshot JSON from full 5-stage pipeline (10 cases).
- Added `proptest` to `sdi-core` and `sdi-pipeline` dev-dependencies.

**bifl-tracker Validation:**
- `tests/fixtures/bifl-tracker-baselines/` (NEW): 5 sdi-py snapshots from bifl-tracker
  at commits b6288d79, 60ef2a1d (×2), b5e43d46, 954d61b0.
- `tools/validate-against-bifl-tracker.sh` (NEW): Shell script that checks out each
  baseline commit, runs sdi-rust, and compares metrics within documented tolerances
  (modularity ≤1%, community count ≤±10%, pattern entropy ≤5%). Includes pure-compute
  parity check using `embed_compute` example.

## Root Cause (bugs only)
**BUG 1:** `parse_repository` returns `Vec<FeatureRecord>` unconditionally — never signals
that extensioned files were skipped. Fix: pipeline checks for extensioned files outside `.sdi/`
after parsing and returns `PipelineError::NoGrammarsAvailable` if present but unparseable.

**BUG 2:** `error_exit_code` in `main.rs` had no `PipelineError` downcast — `NoGrammarsAvailable`
would have mapped to `RuntimeError` (1) instead of `AnalysisError` (3). Fix: added explicit
downcast arm checking for `PipelineError::NoGrammarsAvailable`.

## Files Modified
- `crates/sdi-pipeline/src/pipeline.rs` — emit `NoGrammarsAvailable`; exclude `.sdi/` from check
- `crates/sdi-cli/src/main.rs` — add PipelineError downcast in `error_exit_code`
- `crates/sdi-cli/tests/exit_codes.rs` — update test to expect `.code(3)` for unknown languages
- `crates/sdi-core/src/facade.rs` — fix broken intra-doc links to `Snapshot`/`DivergenceSummary`
- `crates/sdi-parsing/src/lib.rs` — fix broken intra-doc link to `ACTIVE_TREES`
- `crates/sdi-graph/src/dependency_graph.rs` — fix broken intra-doc links to `FeatureRecord`
- `crates/sdi-core/Cargo.toml` — add `proptest` dev-dependency
- `crates/sdi-pipeline/Cargo.toml` — add `proptest` dev-dependency
- `README.md` — expanded (was 58 lines, now 115 lines, under 200)
- `docs/migrating-from-sdi-py.md` — expanded from stub to full carry/change matrix
- `docs/cli-integration.md` (NEW)
- `docs/library-embedding.md` (NEW)
- `docs/determinism.md` (NEW)
- `crates/sdi-cli/examples/embed_pipeline.rs` (NEW)
- `crates/sdi-cli/examples/embed_compute.rs` (NEW)
- `crates/sdi-cli/examples/custom_config.rs` (NEW)
- `crates/sdi-core/tests/prop_thresholds.rs` (NEW)
- `crates/sdi-patterns/tests/prop_normalize.rs` (NEW)
- `crates/sdi-pipeline/tests/prop_pipeline.rs` (NEW)
- `tests/fixtures/bifl-tracker-baselines/` (NEW directory with 5 snapshot JSONs)
- `tools/validate-against-bifl-tracker.sh` (NEW)

## Docs Updated
- `README.md` — expanded with install paths, embedding guide, docs links
- `docs/cli-integration.md` — new
- `docs/library-embedding.md` — new
- `docs/determinism.md` — new
- `docs/migrating-from-sdi-py.md` — expanded from stub to full guide

## Human Notes Status
- Tester BUG: `PipelineError::NoGrammarsAvailable` never emitted — COMPLETED
- Tester BUG: `error_exit_code` doesn't downcast `PipelineError::NoGrammarsAvailable` — COMPLETED

## Observed Issues (out of scope)
- `crates/sdi-config/src/thresholds.rs:46` — `validate_and_prune_overrides` is unused
  (pre-existing dead code warning, not blocking)
- `crates/sdi-graph/src/dependency_graph.rs:9` — unused `tracing::debug` import
  (pre-existing, not blocking)
- `crates/sdi-patterns/src/*.rs` — several unused import warnings (pre-existing)
- `tests/boundary_lifecycle.rs` (workspace root) — comment-only placeholder; Cargo cannot
  run it as a test. Real test is at `crates/sdi-cli/tests/boundary_lifecycle.rs`. Per
  M10 reviewer note, should be removed in a future cleanup pass.
