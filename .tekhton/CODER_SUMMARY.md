# Coder Summary
## Status: COMPLETE

## What Was Implemented
- Implemented real `compute_boundary_violations` in `crates/sdivi-core/src/compute/boundaries.rs`:
  - Compiles boundary globs via `globset`
  - Assigns each node to its most-specific-matching boundary (longest glob wins; ties broken by ascending name)
  - Flags edges where both endpoints are in different boundaries and the source boundary's `allow_imports_from` does not name the target boundary
  - Skips edges with unscoped endpoints (neither endpoint matches any boundary)
  - Returns sorted violation pairs
- Created private helper module `crates/sdivi-core/src/compute/violation.rs`:
  - `CompiledBoundary`: pre-built globset + allow list
  - `compile_boundaries`: compiles all boundary globs once per call
  - `match_boundary`: most-specific-wins matching with name tie-break
- Added `globset` as a direct dep to `sdivi-core/Cargo.toml` (already satisfied via workspace dep)
- Updated `assemble_snapshot` in `sdivi-snapshot` to accept `violation_count: u32` param;
  populated into `IntentDivergenceInfo` when boundary spec is present
- Updated `sdivi-pipeline` to compute violations and thread count through to `assemble_snapshot`
- Added `graph_to_boundary_input` and `spec_to_boundary_input` helpers to `pipeline/helpers.rs`
- Created `crates/sdivi-core/tests/compute_boundary_violations.rs` — 12 unit tests covering:
  empty spec, empty graph, db→api violation, api→db allowed, unscoped nodes, same-boundary
  edges, non-transitivity, most-specific glob, tie-break by name, sorted output
- Moved `snapshot.rs` internal tests to `crates/sdivi-snapshot/tests/snapshot_unit.rs`
- Updated `boundary_spec_assembly.rs` tests to pass `violation_count`
- Updated `boundary_lifecycle.rs` integration test with `snapshot_with_boundary_violations_reports_nonzero_count`
- Updated `wasm_smoke.rs` with boundary violation WASM tests; split oversized file into
  `wasm_smoke.rs` (242 lines, function smoke tests) and `wasm_snapshot.rs` (156 lines,
  snapshot/delta/trend/ADL tests) to stay under the 300-line ceiling
- Updated `CHANGELOG.md` and `docs/cli-integration.md`

## Root Cause (bugs only)
The `compute_boundary_violations` function in `crates/sdivi-core/src/compute/boundaries.rs`
was stubbed to always return `{ violation_count: 0, violations: [] }`. This made Factor 4
(boundary violation velocity) permanently inactive in `sdivi check`.

## Files Modified
- `crates/sdivi-core/src/compute/boundaries.rs` — real implementation replacing stub
- `crates/sdivi-core/src/compute/violation.rs` (NEW) — private glob-matching helpers
- `crates/sdivi-core/src/compute/mod.rs` — added `mod violation`
- `crates/sdivi-core/Cargo.toml` — added `globset` dependency
- `crates/sdivi-core/tests/compute_boundary_violations.rs` (NEW) — 12 unit tests
- `crates/sdivi-snapshot/src/snapshot.rs` — `assemble_snapshot` + `violation_count` param
- `crates/sdivi-snapshot/tests/snapshot_unit.rs` (NEW) — extracted from snapshot.rs inline tests
- `crates/sdivi-snapshot/tests/boundary_spec_assembly.rs` — updated to pass `violation_count`
- `crates/sdivi-snapshot/tests/atomic_write.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/tests/compute_trend.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/tests/delta_pure.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/tests/null_vs_zero.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/tests/retention.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/tests/snapshot_load.rs` — updated `assemble_snapshot` call
- `crates/sdivi-snapshot/src/delta.rs` — minor test adjustment
- `crates/sdivi-snapshot/src/trend.rs` — minor test adjustment
- `crates/sdivi-pipeline/src/pipeline.rs` — compute violations and pass to assemble_snapshot
- `crates/sdivi-pipeline/src/helpers.rs` — `graph_to_boundary_input` and `spec_to_boundary_input`
- `crates/sdivi-pipeline/src/boundaries.rs` — minor update
- `crates/sdivi-cli/tests/boundary_lifecycle.rs` — added violation count integration test
- `bindings/sdivi-wasm/src/exports.rs` — updated boundary violations export
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` — trimmed to 242 lines (imports cleaned)
- `bindings/sdivi-wasm/tests/wasm_snapshot.rs` (NEW) — 156 lines, snapshot/delta/trend/ADL tests
- `CHANGELOG.md` — entry for `compute_boundary_violations` implementation
- `docs/cli-integration.md` — boundary violation rate factor documented

## Docs Updated
- `CHANGELOG.md` — added entry under Unreleased
- `docs/cli-integration.md` — documented boundary_violation_rate threshold factor

## Human Notes Status
N/A — no Human Notes section present in this milestone.

## Observed Issues (out of scope)
- `crates/sdivi-patterns/src/catalog.rs:7` — unused import `sdivi_config::PatternsConfig`
- `crates/sdivi-patterns/src/catalog.rs:10` — unused import `fingerprint_node_kind`
- `crates/sdivi-patterns/src/catalog.rs:11` — unused import `crate::queries`
- `crates/sdivi-graph/src/dependency_graph.rs` — one unused import warning (cargo fix suggestion)
