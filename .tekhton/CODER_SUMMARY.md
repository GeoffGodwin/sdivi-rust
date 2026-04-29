# Coder Summary
## Status: COMPLETE

## What Was Implemented

### Bug Fix
- Fixed `crates/sdi-detection/src/leiden/cpm.rs` — removed doc test that referenced private module `cpm` via the external path `sdi_detection::leiden::cpm::cpm_move_gain`; replaced with `#[cfg(test)]` unit tests. Doc tests in `sdi-detection` now all pass.

### Simple Blocker Fix (Reviewer Report)
- Added required escalation to `HUMAN_ACTION_REQUIRED.md` under `## Action Items`:  doc comment placement rule that has recurred across three milestone runs.

### Milestone 7: Snapshot Assembly, Delta, and Persistence

**sdi-snapshot crate** (`crates/sdi-snapshot/src/`):
- `snapshot.rs`: `Snapshot` struct (`snapshot_version = "1.0"`, `timestamp`, `commit`, `graph: GraphMetrics`, `partition: LeidenPartition`, `catalog: PatternCatalog`, `intent_divergence: Option<IntentDivergenceInfo>`), `IntentDivergenceInfo`, `build_snapshot()` pure assembly function, `Snapshot::load()` — 272 lines
- `delta.rs`: `DivergenceSummary` with all 4 `Option<_>` fields (no `skip_serializing_if` → explicit JSON null), `compute_delta()` pure function, `null_summary()` — 291 lines
- `store.rs`: `write_snapshot()` atomic tempfile-in-same-dir + rename, `iso_to_filename_safe()`, blake3 hash for filename — 107 lines
- `retention.rs`: `enforce_retention()` lexicographic sort → delete oldest — 136 lines
- `lib.rs`: updated to declare and re-export all four modules

**sdi-core crate** (`crates/sdi-core/src/`):
- `pipeline.rs`: `Pipeline::new(config, adapters)` (O(1)), `Pipeline::snapshot(repo_root, commit, timestamp)` (all 5 stages + atomic write + retention), `Pipeline::delta(prev, curr)` (pure static function) — 185 lines
- `lib.rs`: added `pub mod pipeline`, `pub use pipeline::Pipeline`, re-exports of Snapshot/DivergenceSummary/etc. from sdi-snapshot
- `error.rs`: added `SnapshotIo(std::io::Error)` variant (no `#[from]` to avoid conflict with `Io`)
- `Cargo.toml`: added `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-snapshot`, `tracing`

**sdi-cli commands** (`crates/sdi-cli/src/`):
- `commands/snapshot.rs`: `sdi snapshot [--commit REF] [--format json|text]` — Richards algorithm for ISO 8601 timestamp without external date crates — 119 lines
- `commands/diff.rs`: `sdi diff <prev> <curr> [--format json|text]` — loads two snapshots, calls `Pipeline::delta`, version-compat warning per Rule 17 — 51 lines
- `commands/mod.rs`: added `pub mod diff; pub mod snapshot;`
- `main.rs`: added `Snapshot` and `Diff` subcommands with clap; wired in match
- `output/json.rs`: added `print_snapshot`, `print_divergence`
- `output/text.rs`: added `print_snapshot`, `print_divergence`

**Dependency additions**:
- `sdi-snapshot/Cargo.toml`: added `sdi-graph`, `sdi-detection`, `sdi-patterns`, `blake3`, `tempfile`, `tracing`
- `sdi-core/Cargo.toml`: added `sdi-parsing`, `sdi-graph`, `sdi-detection`, `sdi-snapshot`, `tracing`

**Derive additions** (required by `Snapshot: PartialEq`):
- `sdi-graph/src/metrics.rs`: added `PartialEq` to `GraphMetrics` derive
- `sdi-patterns/src/catalog.rs`: added `PartialEq` to `PatternLocation`, `PatternStats`, `PatternCatalog`

**Test files** (all new):
- `crates/sdi-snapshot/tests/atomic_write.rs`: 4 tests — write creates file, valid JSON, path in target dir (not /tmp), no temp files left
- `crates/sdi-snapshot/tests/retention.rs`: 4 tests — keeps newest N, deletes oldest, max=0 unlimited, no-op when under limit
- `crates/sdi-snapshot/tests/null_vs_zero.rs`: 4 tests — null_summary is all-None, identical snaps have zero delta, JSON null is explicit, JSON 0.0 is explicit
- `crates/sdi-snapshot/tests/delta_pure.rs`: 1 proptest — `compute_delta` is referentially transparent
- `crates/sdi-cli/tests/snapshot_diff.rs`: 5 integration tests — snapshot exits 0, two snapshots create 2 files, JSON format valid, diff exits 0, diff JSON has coupling_delta field
- `sdi-snapshot/Cargo.toml` dev-deps: added `proptest`, `sdi-graph`, `sdi-detection`, `sdi-patterns`

**Version test fix**:
- `crates/sdi-cli/tests/version.rs`: updated expected version from `0.0.6` to `0.0.7`

## Root Cause (bugs only)
- `cpm.rs:22`: `mod cpm` inside `leiden/mod.rs` is `mod cpm` (not `pub mod`), so the doc test path `sdi_detection::leiden::cpm::cpm_move_gain` fails to compile at the doc-test external boundary. Fix: removed doc test, added `#[cfg(test)]` unit tests.

## Files Modified
- `.tekhton/HUMAN_ACTION_REQUIRED.md` — added doc comment placement rule escalation
- `crates/sdi-detection/src/leiden/cpm.rs` — removed invalid doc test, added unit tests (BUG fix)
- `crates/sdi-snapshot/src/snapshot.rs` (NEW)
- `crates/sdi-snapshot/src/delta.rs` (NEW)
- `crates/sdi-snapshot/src/store.rs` (NEW)
- `crates/sdi-snapshot/src/retention.rs` (NEW)
- `crates/sdi-snapshot/src/lib.rs`
- `crates/sdi-snapshot/Cargo.toml`
- `crates/sdi-snapshot/tests/atomic_write.rs` (NEW)
- `crates/sdi-snapshot/tests/retention.rs` (NEW)
- `crates/sdi-snapshot/tests/null_vs_zero.rs` (NEW)
- `crates/sdi-snapshot/tests/delta_pure.rs` (NEW)
- `crates/sdi-core/src/pipeline.rs` (NEW)
- `crates/sdi-core/src/lib.rs`
- `crates/sdi-core/src/error.rs`
- `crates/sdi-core/Cargo.toml`
- `crates/sdi-cli/src/commands/snapshot.rs` (NEW)
- `crates/sdi-cli/src/commands/diff.rs` (NEW)
- `crates/sdi-cli/src/commands/mod.rs`
- `crates/sdi-cli/src/main.rs`
- `crates/sdi-cli/src/output/json.rs`
- `crates/sdi-cli/src/output/text.rs`
- `crates/sdi-cli/tests/snapshot_diff.rs` (NEW)
- `crates/sdi-cli/tests/version.rs`
- `crates/sdi-graph/src/metrics.rs` — added `PartialEq` derive
- `crates/sdi-patterns/src/catalog.rs` — added `PartialEq` derives

## Architecture Decisions
- `Pipeline::new(config, adapters)` — adapters injected by composition root (sdi-cli), keeping sdi-core free of direct language adapter crate deps
- `Snapshot` embeds `GraphMetrics` (full graph summary in JSON output)
- `DivergenceSummary` uses `Option<f64>` without `skip_serializing_if` → explicit JSON null (per Watch For)
- `Snapshot.intent_divergence` uses `skip_serializing_if = "Option::is_none"` → completely absent from JSON when no BoundarySpec (absent ≠ null)
- Timestamp is an explicit parameter to `build_snapshot()` enabling deterministic testing
- File naming: `snapshot_YYYYMMDDTHHMMSS_<hash8>.json` (blake3 of JSON content)
- Retention enforced AFTER successful atomic rename (per Watch For note)
- tempfile created IN snapshot dir via `NamedTempFile::new_in(dir)`, not /tmp (per Watch For note)
- `unix_to_iso8601` implemented via Richards algorithm — no external date crate needed

## Test Results
- 307 tests pass excluding sdi-detection's pre-existing proptest overflow bug
- 1 pre-existing failure: `prop_any_seed_deterministic` — arithmetic overflow at `leiden/modularity.rs:102` with seed `2860851616` (present before M07, tracked in .tekhton/)

## Human Notes Status
- HUMAN_ACTION_REQUIRED.md escalation: COMPLETED

## Docs Updated
Public surface changes:
- `sdi snapshot` and `sdi diff` CLI commands are new public surface. These are documented via `--help` (clap-derived) and the `# Examples` blocks in `Pipeline::snapshot` and `Pipeline::delta` rustdoc.
- `Snapshot`, `DivergenceSummary`, `Pipeline` are new public types re-exported from `sdi-core`. All have doc comments with `# Examples` blocks.
- No existing `docs/` pages updated because `docs/cli-integration.md` does not yet exist (Milestone 8-10 deliverable per CLAUDE.md).
