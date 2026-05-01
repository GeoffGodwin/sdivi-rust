# Coder Summary
## Status: COMPLETE

## What Was Implemented
- `crates/sdi-pipeline/src/commit_extract.rs` (NEW) — ref resolution, UTC date normalization, tree extraction via git archive | tar
- `crates/sdi-pipeline/src/error.rs` — added `CommitExtract(CommitExtractError)` variant  
- `crates/sdi-pipeline/src/helpers.rs` (NEW) — extracted `build_edge_weight_map` and `compute_path_partition` from pipeline.rs (needed to stay under 300-line ceiling)
- `crates/sdi-pipeline/src/pipeline.rs` — branched `snapshot_with_mode` on `commit.is_some()`: resolve SHA, get UTC commit date, extract tree to tempdir, pass parse_root to parsing stage, pass `ending_at` to change-coupling, override timestamp
- `crates/sdi-pipeline/src/lib.rs` — added `commit_extract` module and re-export
- `crates/sdi-pipeline/tests/commit_snapshot.rs` (NEW) — 3-commit fixture tests
- `crates/sdi-cli/src/commands/snapshot.rs` — error display note for CommitExtract
- `crates/sdi-cli/tests/exit_codes.rs` — added --commit nonexistent and --commit non-git-dir cases
- `crates/sdi-cli/tests/historical_commit_lifecycle.rs` (NEW) — full CLI integration test
- `tests/historical_commit_lifecycle.rs` (NEW placeholder) — workspace layout placeholder
- `docs/cli-integration.md` — new "Analyzing a historical commit" section
- `CHANGELOG.md` — entry for M16

## Root Cause (bugs only)
N/A — this is a feature milestone.

## Files Modified
- `crates/sdi-pipeline/src/commit_extract.rs` (NEW)
- `crates/sdi-pipeline/src/helpers.rs` (NEW)
- `crates/sdi-pipeline/src/error.rs` (MODIFIED)
- `crates/sdi-pipeline/src/pipeline.rs` (MODIFIED)
- `crates/sdi-pipeline/src/lib.rs` (MODIFIED)
- `crates/sdi-pipeline/tests/commit_snapshot.rs` (NEW)
- `crates/sdi-cli/src/commands/snapshot.rs` (MODIFIED)
- `crates/sdi-cli/tests/exit_codes.rs` (MODIFIED)
- `crates/sdi-cli/tests/historical_commit_lifecycle.rs` (NEW)
- `tests/historical_commit_lifecycle.rs` (NEW placeholder)
- `docs/cli-integration.md` (MODIFIED)
- `CHANGELOG.md` (MODIFIED)

## Human Notes Status
No Human Notes section present in task description.

## Docs Updated
- `docs/cli-integration.md` — new "Analyzing a historical commit" section; updated `sdi snapshot` flags table
- `docs/library-embedding.md` — added note clarifying `--commit REF` is pipeline/CLI convenience only

## Observed Issues (out of scope)
- Pre-existing test failure: `workspace_version_is_0_1_0` in `crates/sdi-cli/tests/workspace_version.rs` checks for version `"0.1.0"` but workspace is at `"0.1.2"`. Already failing before M16.
