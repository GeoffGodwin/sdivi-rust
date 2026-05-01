# Reviewer Report — Milestone 10: Boundaries — Infer, Ratify, Show
Review cycle: 1 of 4

## Verdict
APPROVED_WITH_NOTES

## ACP Verdicts
- ACP: Add `path_partition: BTreeMap<String, u32>` to `Snapshot` — ACCEPT. Backward-compatible (`serde(default, skip_serializing_if)`). Correctly populated in `sdi-pipeline`, never in `sdi-core` (WASM cleanliness maintained). `assemble_snapshot` returns empty map; pipeline mutates after assembly. Clean seam.

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdi-pipeline/src/pipeline.rs:249` — `comm_id as u32` casts `usize` community IDs from `LeidenPartition.assignments`. Same theoretical-truncation category as the pre-existing security finding. Practical community counts will never exceed `u32::MAX`, but `u32::try_from(comm_id).unwrap_or(u32::MAX)` or a saturating cast would be more defensive.
- `crates/sdi-pipeline/src/store.rs:118` — `trimmed.contains(" #")` in the comment-detection heuristic will fire on YAML values that incidentally contain ` #` (hex colors, URL fragments, etc.). This is a warning-only path (KDD-6 accepts comment loss), but a false-positive warning is user-confusing. Limiting the check to `trimmed.starts_with('#')` is safer and still catches the common case.
- `crates/sdi-cli/src/commands/boundaries.rs:89` — `modules: p.node_ids.clone()` writes exact file paths into `BoundaryDef.modules`, which is documented as a glob-pattern field. Exact paths are technically valid (match only themselves), but users editing the output manually may be confused by the absence of wildcards.
- `crates/sdi-cli/tests/boundaries_stub.rs` — filename says "stub" but the tests are now full integration tests. Rename to `boundaries_empty_repo.rs` in a future cleanup pass.
- `crates/sdi-cli/tests/version.rs:14` — Hardcoded `"0.0.11"` version string (pre-existing, carried from M09 note). Replace with `env!("CARGO_PKG_VERSION")` when touching this file next.

## Coverage Gaps
- `write_boundary_spec` comment-loss warning path (`store.rs:108-128`) has no test verifying the `eprintln!` fires when an existing file contains `#` lines.
- `compute_path_partition` (`pipeline.rs:239-252`) is a private helper with no unit test. A test constructing a small `DependencyGraph` + `LeidenPartition` and asserting path→community mapping would protect the numeric cast and the UTF-8 path filter.
- `tests/boundary_lifecycle.rs` at workspace root is a comment-only placeholder that Cargo cannot run as an integration test. It provides no protection; consider removing in favour of the real test at `crates/sdi-cli/tests/boundary_lifecycle.rs`.

## Drift Observations
- `crates/sdi-pipeline/src/store.rs` — `read_snapshots` and `latest_snapshot` each implement identical directory-listing and sorting logic (filter `snapshot_*.json`, sort by filename). Extract to a private `list_snapshot_entries(dir)` helper to eliminate the duplication.
- `crates/sdi-snapshot/src/boundary_inference.rs:152` — `count_stable_tail` comment says "Walk pairs from newest-1 backwards" but the loop checks individual preceding partitions, not explicit pairs. Semantics are correct; the comment is slightly misleading.
- `crates/sdi-cli/src/commands/boundaries.rs:93` — Ratified `BoundarySpec` sets `version: None`. Auto-ratified specs could set `version: Some("1.0".to_string())` for forward-compat explicitness, consistent with `SNAPSHOT_VERSION`.
