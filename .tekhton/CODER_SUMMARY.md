# Coder Summary
## Status: COMPLETE

## What Was Implemented
### Milestone 10: Boundaries — Infer, Ratify, Show

**Architecture Decision:**
Added `path_partition: BTreeMap<String, u32>` to `Snapshot`. `LeidenPartition.assignments`
uses numeric node indices, but `PriorPartition.cluster_assignments` requires string paths.
This additive field (with `serde(default, skip_serializing_if)`) stores the path→community
mapping computed from the `DependencyGraph` + `LeidenPartition` during pipeline execution,
enabling boundary inference from snapshot history without re-running the pipeline. Old
snapshots with missing field produce empty proposals (normal degraded behavior).

- `sdi-snapshot`: Added `path_partition: BTreeMap<String, u32>` to `Snapshot`
- `sdi-pipeline::pipeline`: Computes path_partition from DependencyGraph + LeidenPartition
- `sdi-config::boundary`: Added `BoundarySpec::to_yaml()` pure serializer
- `sdi-pipeline::store`: Added `write_boundary_spec()` with atomic write + comment-loss detection
- `sdi-pipeline::boundaries` (NEW): `read_prior_partitions`, `infer_from_snapshots`
- `sdi-pipeline::lib`: Exported boundaries module
- `sdi-cli::commands::boundaries`: Full infer/ratify/show implementation
- Updated `boundaries_stub.rs` to match new behavior (no longer stubs)
- New tests: boundary_roundtrip, boundaries_show, infer_boundaries, boundary_lifecycle
- `docs/migrating-from-sdi-py.md` (NEW stub with comment-loss section)
- Fixed unused import warnings in `crates/sdi-core/tests/infer_boundaries.rs` and
  `crates/sdi-pipeline/src/boundaries.rs`

## Root Cause (bugs only)
N/A — feature implementation

## Files Modified
- `crates/sdi-snapshot/src/snapshot.rs` — added path_partition field
- `crates/sdi-pipeline/src/pipeline.rs` — compute path_partition
- `crates/sdi-config/src/boundary.rs` — added to_yaml() method
- `crates/sdi-pipeline/src/store.rs` — added write_boundary_spec()
- `crates/sdi-pipeline/src/boundaries.rs` (NEW) — read_prior_partitions, infer_from_snapshots
- `crates/sdi-pipeline/src/lib.rs` — export boundaries module
- `crates/sdi-cli/src/commands/boundaries.rs` — full implementation
- `crates/sdi-cli/tests/boundaries_stub.rs` — updated for real behavior
- `crates/sdi-config/tests/boundary_roundtrip.rs` (NEW)
- `crates/sdi-cli/tests/boundaries_show.rs` (NEW)
- `crates/sdi-cli/tests/boundary_lifecycle.rs` (NEW)
- `crates/sdi-core/tests/infer_boundaries.rs` (NEW)
- `tests/boundary_lifecycle.rs` (NEW — placeholder comment pointing to the real test)
- `docs/migrating-from-sdi-py.md` (NEW)

## Docs Updated
- `docs/migrating-from-sdi-py.md` — created with comment-loss section (M10 deliverable; full guide in M11)

## Human Notes Status
- Non-blocking note from reviewer: `crates/sdi-cli/tests/version.rs:14` hardcoded version — NOT_ADDRESSED (out of scope for M10)

## Architecture Change Proposals

### Add `path_partition` field to `Snapshot`
- **Current constraint**: `Snapshot` stores `LeidenPartition` with numeric node indices; `PriorPartition` for `infer_boundaries` requires string paths.
- **What triggered this**: No existing mechanism maps numeric partition indices to file paths in stored snapshots. Reading snapshots for boundary inference would produce empty `PriorPartition` values without this mapping.
- **Proposed change**: Added `path_partition: BTreeMap<String, u32>` to `Snapshot`. Set during pipeline execution from `DependencyGraph` + `LeidenPartition`. `#[serde(default, skip_serializing_if = "BTreeMap::is_empty")]` preserves backward compat with existing snapshots.
- **Backward compatible**: Yes — old readers skip unknown fields; old snapshots deserialize with empty `path_partition`.
- **ARCHITECTURE.md update needed**: Yes — mention `path_partition` under Snapshot schema in the data flow section.

## Observed Issues (out of scope)
- `crates/sdi-cli/src/commands/catalog.rs`: Has its own local `all_adapters()` duplicating the one in `commands/mod.rs` (noted in M09 CODER_SUMMARY)
