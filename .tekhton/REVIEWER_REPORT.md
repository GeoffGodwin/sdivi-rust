# Reviewer Report — M34: Multi-Category Call-Expression Dispatch Framework
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdivi-patterns/src/queries/mod.rs:280` — Assertion message "logging is catalog-only in v0 for category_for_node_kind; routing for ({kind}, {lang}) would steal from data_access/resource_management" is stale phrasing from M30. Tested behaviour is correct; the message is misleading. Pre-existing, previously flagged by M23 reviewer, deferred again.

## Coverage Gaps
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs:25-37` (`all_matching_categories`) — Hardcoded to P1/P8/P9. When M35+ adds a category to `CALL_DISPATCH`, this function won't detect overlaps with the new category and `no_undocumented_overlaps_in_corpus` silently under-checks. The file-header doc warns about this, but a `// TODO(M35): extend when adding P2` comment at the function body would be more actionable for the implementer.

## Drift Observations
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs:168-174` — The `loser_matches` match in `known_overlaps_winner_matches_dispatch_order` is intentionally hardcoded; the `other => panic!` arm forces future milestones to extend it. Valid design, but the intent isn't commented — a new contributor may read it as incomplete code. Low risk.
- `docs/pattern-categories.md` + Go corpus — `fmt.Errorf("msg")` is classified as `logging` via `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)`. `fmt.Errorf` constructs an error value; it does not emit output. M33 inheritance surfaced by the new corpus. Not M34's to fix, but the eventual Go error-handling pass will need to revisit this regex entry.
- Pre-existing (not introduced by M34): `wasm_package_json_version_matches_workspace` test failure — wasm `package.json` stranded at 0.2.23 while workspace is at 0.2.24.
