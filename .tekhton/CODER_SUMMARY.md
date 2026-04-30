# Coder Summary
## Status: COMPLETE

## What Was Implemented
### From Prior Run (attempt 1/3)
- Workspace Cargo.toml: added sdi-pipeline member/dep, chrono workspace dep
- sdi-config: added `loader` feature gate, renamed validate_and_prune_overrides → validate_overrides_format (no pruning), removed today_iso8601/is_expired
- sdi-graph: added pipeline-records feature, added DependencyGraph::from_node_edges constructor
- sdi-detection: added pipeline-records feature, removed tempfile from runtime, moved FS out of warm_start.rs
- sdi-patterns: added pipeline-records feature, added normalize.rs with normalize_and_hash, updated fingerprint_node_kind
- sdi-snapshot: added pipeline-records feature, renamed build_snapshot→assemble_snapshot, added PatternMetricsResult, convention_drift_delta, trend.rs, boundary_inference.rs
- sdi-pipeline (NEW): Pipeline orchestration crate with cache.rs, store.rs
- sdi-core: reshaped to pure-compute facade with input.rs, compute/ modules, facade.rs
- sdi-cli: updated to use sdi-pipeline::Pipeline
- Tests: added validate_node_id, normalize_and_hash, compute_pattern_metrics, compute_thresholds_check, compute_trend, infer_boundaries, pipeline_smoke tests
- CHANGELOG.md: added M08 entries

### This Run (attempt 2/3)
- sdi-cli/tests/version.rs: updated version string from 0.0.7 → 0.0.8
- sdi-config/Cargo.toml: added `chrono` as optional dep under `loader` feature (clock feature for today's date), made `dirs` optional under `loader`
- sdi-config/src/thresholds.rs: restored `validate_and_prune_overrides(table, today)` which actually prunes expired overrides in-place; added `today_iso8601()` using chrono (feature-gated on `loader`)
- sdi-config/src/load.rs: updated call to use `validate_and_prune_overrides(&mut base, &today_iso8601())`
- sdi-detection/src/leiden/mod.rs: fixed pre-existing subtraction overflow bug — community IDs are now offset by `n` before creating `ModularityState` to prevent node-index / community-ID collision in `remove_node`'s singleton slot; extracted `compute_stability` and `compute_modularity` to `quality.rs` to stay under 300-line ceiling
- sdi-detection/src/leiden/quality.rs (NEW): extracted `compute_stability` and `compute_modularity` from `mod.rs`

## Root Cause (bugs only)
1. **version test**: Prior run incremented crate version to 0.0.8 but test still checked for 0.0.7.
2. **threshold pruning regression**: Prior run changed `validate_and_prune_overrides` to `validate_overrides_format` (no pruning) but integration tests assert expired overrides are pruned at load time per CLAUDE.md rules.
3. **Leiden subtraction overflow**: Pre-existing bug since M05. In `local_move_phase`, when `ModularityState::from_assignment` is called with a partition whose community IDs overlap with node indices (0..k), `remove_node(X)` writes `size[X] = 1` to the singleton slot. If community X had multiple members, this corrupts `size[X]`. Later `remove_node` calls on those members decrement from the corrupted-lower value, eventually reaching 0 and underflowing. Fix: offset all partition community IDs by `n` before building state, so singleton slots (node indices 0..n) never overlap with community slots (n..n+k).

## Files Modified
- `crates/sdi-cli/tests/version.rs` — version string 0.0.7→0.0.8
- `crates/sdi-config/Cargo.toml` — chrono optional dep (loader feature), dirs optional
- `crates/sdi-config/src/thresholds.rs` — restored pruning; today_iso8601() with chrono
- `crates/sdi-config/src/load.rs` — call validate_and_prune_overrides with today
- `crates/sdi-detection/src/leiden/mod.rs` — community ID offset fix; use quality module
- `crates/sdi-detection/src/leiden/quality.rs` (NEW) — compute_stability, compute_modularity

## Docs Updated
None — no public-surface changes in this task (all changes are bug fixes to internal behavior).

## Human Notes Status
No Human Notes present in this task.

## Observed Issues (out of scope)
- `sdi-detection/src/leiden/modularity.rs:105`: the `sigma_tot[node] = graph.degree[node]` and `size[node] = 1` assignments in `remove_node` still assume the singleton slot is clean. The offset fix in `local_move_phase` guarantees this for the primary call site, but any future caller of `remove_node` that doesn't offset must be aware of this constraint. A doc comment on `remove_node` documenting the precondition (community IDs must not overlap with node indices) would help.
