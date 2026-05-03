# Reviewer Report — M19: compute_boundary_violations implementation

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `helpers.rs:61` — `graph_to_boundary_input` uses `.unwrap_or_default()` for nodes without a path, producing empty-string node IDs. If `validate_node_id("")` rejects empty strings, the whole `compute_boundary_violations` call returns `Err`, which `pipeline.rs:238` swallows via `unwrap_or_else` with a warning and sets `violation_count = 0`. Behavior is safe but silent; worth a comment explaining the intent.
- `snapshot.rs:175` — The `#[allow(clippy::too_many_arguments)]` justification comment meets Rule 20, but the comment is placed _above_ the `#[allow]` line rather than as an inline `//` on the attribute. Clippy won't flag this, but the convention used elsewhere is the `// justification` comment style on the same line as the `#[allow]`.

## Coverage Gaps
- `crates/sdivi-core/tests/boundary_violations_proptest.rs` appears in `git status` as untracked but is not listed in CODER_SUMMARY files. If it contains property tests for `compute_boundary_violations`, it should be staged and included in this changeset; if it is a stub/placeholder, either remove it or include it as a tracked file.
- Duplicate-edge behavior is untested: if `graph.edges` contains the same `(source, target)` pair twice, `violations` will have two entries for a single conceptual violation. The graph layer likely prevents this but there is no test asserting it.

## Drift Observations
- `crates/sdivi-pipeline/src/helpers.rs:55-70` — `graph_to_boundary_input` calls `dg.node_path(i)` in a `(0..n)` index range loop rather than iterating nodes directly. This pattern assumes `DependencyGraph` has a contiguous `0..node_count()` index space. If `DependencyGraph` ever uses non-contiguous indices (e.g. after node removal), this silently drops nodes. Consistent with current usage elsewhere but worth a `// SAFETY:` note on the assumption.
- `bindings/sdivi-wasm/src/exports.rs:165-184` — `assemble_snapshot` WASM wrapper passes `violation_count = 0` to the core function then manually overwrites `snap.intent_divergence` if `boundary_count` is `Some`. This is a second code path that assembles `IntentDivergenceInfo` outside of `sdivi_snapshot::assemble_snapshot`. Currently harmless but diverges from the single-assembly-seam principle over time.

## ACP Verdicts
(No `## Architecture Change Proposals` section found in CODER_SUMMARY.md — section omitted.)
