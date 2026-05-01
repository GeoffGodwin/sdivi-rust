# Reviewer Report — M15: Change-Coupling Analyzer
_Cycle 2 of 4 | Branch: milestones/v0_

## Verdict
APPROVED_WITH_NOTES

## Prior Blocker Resolution

**Simple Blocker (Cycle 1): `bindings/sdi-wasm/src/exports.rs:174` — `as usize` cast on `violation_count`**
- Status: **FIXED**
- Evidence: Line 174 now reads `violation_count: input.violation_count.unwrap_or(0),` — the `as usize` cast is gone; `unwrap_or(0)` on `Option<u32>` produces `u32` matching the field type exactly.

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdi-core/src/compute/change_coupling.rs:96-100` — the `if a < b { (a, b) } else { (b, a) }` branch is dead code; `sorted_files` is already sorted so `i < j` guarantees `sorted_files[i] < sorted_files[j]`; the else arm never executes. Write `(a.clone(), b.clone())` directly.
- `crates/sdi-pipeline/src/change_coupling.rs:101-103,123` — double empty-SHA guard: the inner `if !sha.is_empty()` at line 123 is always true when reached because line 101 already continues past empty SHAs.
- `bindings/sdi-wasm/src/exports.rs:168` — WASM `assemble_snapshot` hardcodes `change_coupling: None`; WASM consumers who call `compute_change_coupling` cannot include the result in an assembled snapshot. Track for a follow-up field addition to `WasmAssembleSnapshotInput`.
- `bindings/sdi-wasm/src/types.rs:53-58` — `WasmLeidenConfigInput` has no `edge_weights` field; WASM consumers calling `detect_boundaries` always get unweighted Leiden. Intentional for MVP but untracked.
- `crates/sdi-core/src/input/types.rs:145` — `LeidenConfigInput::edge_weights` is `Option<BTreeMap<(String, String), f64>>`; `serde_json` cannot serialize tuple-keyed maps; Rust embedders that call `serde_json::to_value` on a populated `edge_weights` will get a runtime serialization error. Consider a `"source\x1ftarget"` delimited string key via a newtype or `serde_as`.
- `crates/sdi-core/src/compute/change_coupling.rs:83` — `all_files: HashSet<String>` used only for `.len()`; mildly inconsistent with the project's `BTreeSet` convention.

## Coverage Gaps
- `compute_change_coupling`: no test for the window-truncation path when `events.len() > history_depth` with a non-zero `history_depth`.
- No test that round-trips `LeidenConfigInput` with a populated `edge_weights` through `serde_json::to_value` — would surface the tuple-key serialization error noted above.
- No WASM integration test for `assemble_snapshot` with `violation_count` set.

## Drift Observations
- `[bindings/sdi-wasm/src/types.rs vs crates/sdi-core/src/input/types.rs]` — `WasmLeidenConfigInput` silently diverged from `LeidenConfigInput` in M15 (core gained `edge_weights`; WASM wrapper did not). The `to_core` deserialization silently defaults to `None`, so no crash, but the divergence will widen as fields are added without mirrored updates to the WASM wrapper.
- `[crates/sdi-pipeline/src/change_coupling.rs:86-90]` — `parse_git_log_output` doc comment describes newline-separated filenames but the actual format is NUL-terminated (`-z` flag). The parsing logic is correct; the prose is wrong.
- `[crates/sdi-detection/src/leiden/quality.rs:1]` — all other rewritten `leiden/` submodules have a `//!` module-level doc comment; `quality.rs` does not. Minor consistency gap.
