# Reviewer Report — M40: `collection_pipelines` pattern category

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
- `CODER_SUMMARY.md` marks the wasm_smoke.rs coverage gap (`resource_management`, `state_management`, `type_assertions`) as NOT_ADDRESSED, but `wasm_smoke.rs` now asserts all three names in `list_categories_returns_schema_version_and_expected_count`. The summary appears to have been written before the test update was finalised and not reconciled — code is correct, summary is stale.
- `crates/sdivi-patterns/src/queries/mod.rs:123-124` — no blank line between closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. Pre-existing; not introduced by M40.
- `crates/sdivi-patterns/src/queries/mod.rs:31-33` (doc for `category_for_node_kind`) — still only lists `logging` as the callee-only category; `collection_pipelines`, `framework_hooks`, `schema_validation`, and `state_store` are also callee-only. Pre-existing gap grown by M40.

---

## Coverage Gaps
- None

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:123-124` — the missing blank line between the `CALL_DISPATCH` array close and the `classify_hint` doc block has been flagged by the M38 and M39 reviewers and remains unresolved. Accumulating drift suggests a dedicated cleanup pass rather than per-milestone carry-through.
- `crates/sdivi-patterns/src/queries/tests.rs:259-267` — test `null_safety_node_kinds_do_not_match_non_ts_js_languages` has an inverted name (body asserts `Some("null_safety")` matches for all languages, i.e. a match, not a non-match). Carry-over from M37; still unresolved.
