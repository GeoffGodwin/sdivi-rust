# Reviewer Report — M37: `null_safety` pattern category

**Review cycle:** 1 of 4
**Reviewer:** code-review agent

---

## Verdict
APPROVED_WITH_NOTES

---

## Complex Blockers (senior coder)
- None

---

## Simple Blockers (jr coder)
- None

---

## Non-Blocking Notes
- `null_safety.rs` public constant doc, module doc, `categories.rs` description, `docs/pattern-categories.md` canonical table, and `MIGRATION_NOTES.md` all list `fn?.()` as an `optional_chain` example — but the coder's own test (`optional_chain_member_access_variants_captured`) proves the grammar emits `call_expression` for optional calls, not `optional_chain`. The code is correct; the docs give a misleading example in four public-facing locations. Suggest removing `fn?.()` from all four example lists in a follow-up pass.
- Test name `null_safety_node_kinds_do_not_match_non_ts_js_languages` (`queries/tests.rs:164`) is semantically inverted: the body asserts `Some("null_safety")` (a match) for `["rust", "python", "go", "java"]`, while the name implies no-match. The assertion is correct and the inline message explains why; only the name misleads.

---

## Coverage Gaps
- No test asserts the per-node count for a chained expression (`a?.b?.c`). The module doc and migration notes claim this produces multiple `optional_chain` nodes (each counted independently), but no test constructs such a chain and verifies the expected count. A dedicated assertion would pin the documented counting semantics.

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:122` — `classify_hint` doc comment reads "(P1/P6/P8/P9 active at M35)"; now at M37. Pre-existing; accumulating across milestones.
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs:26` — "At M35, P1/P6/P8/P9 are active" comment is stale (observed across M36.1, M36.2, M37). One-line fix deferred each cycle.
- `fn?.()` described as `optional_chain` is consistent across `null_safety.rs`, `categories.rs`, `docs/pattern-categories.md`, and `MIGRATION_NOTES.md` — consistent with each other and the milestone spec, but collectively wrong per the grammar. The test is the accurate record; the docs will mislead until corrected.
