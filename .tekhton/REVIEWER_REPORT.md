# Reviewer Report — feature/MorePatterns

**Date:** 2026-06-02
**Review type:** Drift resolution (cycle 1 of 2)
**Task:** Resolve 2 unresolved architectural drift observations in `.tekhton/DRIFT_LOG.md`

---

## Verdict

APPROVED_WITH_NOTES

## Complex Blockers (senior coder)

None

## Simple Blockers (jr coder)

None

## Non-Blocking Notes

- `go_concurrency_node_kind.rs:97–122` — `go_statement_not_misclassified` enumerates 18 `assert_ne` categories then re-asserts `Some("concurrency")`. This is already covered by `go_statement_maps_to_concurrency_category` and `all_concurrency_node_kinds_are_classified`. Redundant but harmless; the negative list will silently drift if new categories are added — consider driving from `ALL_CATEGORIES.iter().filter(|&&c| c != "concurrency")` on a future pass.
- `go_concurrency_node_kind.rs:36–53` — `go_statement_language_parameter_ignored` names and tests a behavior that the doc calls "reserved for future per-language overrides." The test will need updating when that override lands. A one-line comment "asserts current no-op behavior; revisit when per-language dispatch is implemented" would make the intent explicit without blocking.

## Coverage Gaps

None

## Drift Observations

- `crates/sdivi-patterns/tests/go_concurrency_node_kind.rs:97–122` — The negative-category enumeration in `go_statement_not_misclassified` is a manual list parallel to `queries::ALL_CATEGORIES`. The same pattern was just fixed in `all_concurrency_node_kinds_are_classified` (drift observation 2 of this cycle). Both lists will drift independently when new categories are added; worth a single follow-up pass to drive both from `ALL_CATEGORIES`.
