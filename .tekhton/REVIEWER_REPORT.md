# Reviewer Report — M30: Pattern Category `logging`
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `docs/pattern-categories.md` Go/Java table only shows 2 rows (`data_access`, `logging`); other categories are absent from that sub-section. Pre-existing gap, not introduced by M30 — the new row correctly follows the established pattern. No action needed now, but the table remains a human-review liability on every category milestone.
- `crates/sdivi-patterns/src/queries/mod.rs:35` — `ALL_CATEGORIES` doc-example hard-codes `assert_eq!(ALL_CATEGORIES.len(), 7)` and will need bumping again on M31. Acknowledged ergonomic hazard in the milestone's Seeds Forward section; not actionable until the const is restructured.

## Coverage Gaps
- No integration test runs `Pipeline::snapshot` against `tests/fixtures/simple-typescript` and asserts zero `logging` keys in `pattern_metrics`. The milestone's "Tests" section designates the unit-level `category_for_node_kind_never_returns_logging` as the correct vehicle for this guarantee — the gap is accepted by the milestone design, but a pipeline-level smoke test would add integration depth.

## Drift Observations
- `crates/sdivi-core/src/categories.rs:81-88` — `CATEGORIES` const uses explicit `CATALOG_ENTRIES[0].0` through `CATALOG_ENTRIES[6].0` index references. Each category addition requires manually re-verifying every index. Seeds Forward section in M30 flags this; nominating for cleanup: derive `CATEGORIES` from `CATALOG_ENTRIES` without hardcoded indices so the index-shift risk disappears. (Also observed in M29 review at lines 69-76 — still unresolved.)
