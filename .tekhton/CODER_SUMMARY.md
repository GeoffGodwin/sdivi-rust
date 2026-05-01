# Coder Summary
## Status: COMPLETE

## What Was Implemented

### Milestone 9: Trend, Check, Show — Remaining CLI Commands

**`sdi-pipeline` extensions:**
- `crates/sdi-pipeline/src/store.rs`: Added `read_snapshots`, `latest_snapshot`, and `read_snapshot_by_id` — snapshot directory listing and loading helpers used by the new CLI commands.
- `crates/sdi-pipeline/src/pipeline.rs`: Added `WriteMode::{Persist, EphemeralForCheck}` enum and `snapshot_with_mode` method. The existing `snapshot()` delegates to `snapshot_with_mode(..., WriteMode::Persist)` — fully backward compatible.
- `crates/sdi-pipeline/src/lib.rs`: Re-exported `WriteMode`, `read_snapshots`, `latest_snapshot`, `read_snapshot_by_id`.

**New CLI commands:**
- `sdi check [--no-write] [--format json|text]`: Captures a fresh snapshot, diffs against the most recent prior, routes through `sdi_core::compute_thresholds_check`. Exits 10 on breach, 0 otherwise. First-run (no prior) always exits 0 (null `DivergenceSummary` has no checkable deltas). `--no-write` uses `WriteMode::EphemeralForCheck` — no file created, no retention enforced.
- `sdi trend [--last N] [--format json|text]`: Reads stored snapshots, calls `sdi_core::compute_trend`. With <2 snapshots, prints friendly message to stderr and exits 0. `--last N` larger than available silently clamps.
- `sdi show [<id>] [--format json|text]`: Shows a snapshot by ID (filename stem) or defaults to the lexicographically-last snapshot. `--format json` passes raw Snapshot JSON to stdout.
- `sdi boundaries {infer|ratify|show}`: M09 stubs — each prints "not implemented until M10" to stderr and exits 0.

**CLI infrastructure:**
- `commands/mod.rs`: Added 4 new command modules; moved `all_adapters()` helper here (shared across snapshot/check commands).
- `commands/snapshot.rs`: Updated to use `super::all_adapters()`.
- `main.rs`: Added `Check`, `Trend`, `Show`, `Boundaries` to `Commands` enum. `Check` dispatched before the standard `Result<()>` loop to support exit-10 semantics.
- `output/json.rs`: Added `print_check` (wraps `ThresholdCheckResult` in `{exit_code, exceeded, summary, applied_overrides}` shape) and `print_trend`.
- `output/text.rs`: Added `print_check` and `print_trend` plain-text formatters (no ANSI codes, satisfying `NO_COLOR` acceptance criteria).
- `Cargo.toml` (sdi-cli): Added `chrono` with `clock` feature for `chrono::Local::now().date_naive()` in `check.rs`.

**Test files (all new):**
- `tests/exit_codes.rs` — 12 tests covering init/snapshot/check/trend/show/diff/boundaries exit codes.
- `tests/check_thresholds.rs` — 6 tests: first-run exit 0, no-write snapshot count, write creates 1 file, JSON shape, exit_code matches process exit.
- `tests/stdout_stderr_split.rs` — 4 tests: show/check/trend/diff JSON stdout validity + no stderr JSON contamination.
- `tests/show_format.rs` — 5 tests: JSON parses as Snapshot, no-id picks latest file, specific id selection, no-snapshots error, no stderr JSON.
- `tests/trend_format.rs` — 5 tests: 0/1 snapshot message, JSON shape, --last clamping, --last N tail selection.
- `tests/boundaries_stub.rs` — 3 tests: each boundary subcommand exits 0 and writes only to stderr.
- `tests/no_color.rs` — 4 tests: NO_COLOR=1 suppresses ANSI in show/check/trend.

## Root Cause (bugs only)
N/A — milestone implementation, not a bug fix.

## Files Modified
- `crates/sdi-pipeline/src/store.rs` — added read_snapshots, latest_snapshot, read_snapshot_by_id
- `crates/sdi-pipeline/src/pipeline.rs` — added WriteMode enum, snapshot_with_mode method; snapshot() delegates
- `crates/sdi-pipeline/src/lib.rs` — re-exported WriteMode and store read helpers
- `crates/sdi-cli/Cargo.toml` — added chrono with clock feature
- `crates/sdi-cli/src/commands/mod.rs` — added 4 new modules + all_adapters helper
- `crates/sdi-cli/src/commands/snapshot.rs` — switched to super::all_adapters(); removed local fn
- `crates/sdi-cli/src/commands/check.rs` (NEW)
- `crates/sdi-cli/src/commands/trend.rs` (NEW)
- `crates/sdi-cli/src/commands/show.rs` (NEW)
- `crates/sdi-cli/src/commands/boundaries.rs` (NEW)
- `crates/sdi-cli/src/main.rs` — new Commands variants + check dispatch with exit-code handling
- `crates/sdi-cli/src/output/json.rs` — added print_check, print_trend
- `crates/sdi-cli/src/output/text.rs` — added print_check, print_trend
- `crates/sdi-cli/tests/exit_codes.rs` (NEW)
- `crates/sdi-cli/tests/stdout_stderr_split.rs` (NEW)
- `crates/sdi-cli/tests/check_thresholds.rs` (NEW)
- `crates/sdi-cli/tests/show_format.rs` (NEW)
- `crates/sdi-cli/tests/trend_format.rs` (NEW)
- `crates/sdi-cli/tests/boundaries_stub.rs` (NEW)
- `crates/sdi-cli/tests/no_color.rs` (NEW)

## Human Notes Status
- Non-Blocking Note from Reviewer: `crates/sdi-cli/tests/version.rs:14` hardcoded version string → COMPLETED by prior tester run (already fixed before this session).

## Observed Issues (out of scope)
- `crates/sdi-cli/src/commands/catalog.rs`: Has its own local `all_adapters()` that duplicates the one now in `commands/mod.rs`. Both are correct; refactoring catalog.rs was out of scope for M09.
