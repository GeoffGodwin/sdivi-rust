# Reviewer Report — M42: `testing` pattern category

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
- `crates/sdivi-patterns/src/queries/mod.rs:34-35` — The `ALL_CATEGORIES` doc comment still says "Note: `logging` is classified via `classify_hint` callee-text inspection only" without mentioning `testing`. At M42 there are now seven callee-only categories; the single-category callout is significantly stale and will mislead embedders reading the docs. Pre-existing carry-over widened by this milestone.
- `crates/sdivi-patterns/src/queries/mod.rs:131-132` — No blank line between the closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. Pre-existing carry-over from M38+; Rust will silently re-attach the doc to the const rather than the function in certain tooling. Should be fixed in a standalone clean-up commit.

---

## Coverage Gaps
- No fixture-level integration test exercising the in-scope vs. `scope_exclude`-excluded bucket population path. The milestone spec explicitly required: "Integration: a fixture with tests in-scope vs excluded, asserting the bucket populates/empties accordingly." Unit and contract coverage is solid; the integration path is absent. Not a blocker — the tester handles this — but flagged per spec.

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/tests.rs:292-300` — `null_safety_node_kinds_do_not_match_non_ts_js_languages` has an inverted name: the body asserts `Some("null_safety")` (the node kind *does* match for all languages), contradicting "do not match." Carry-over from M37; still unresolved at M42.
- `crates/sdivi-core/src/categories.rs` / `mod.rs` — `category_for_node_kind` doc comment lists only `logging` as callee-only. Now seven categories (`logging`, `framework_hooks`, `schema_validation`, `state_store`, `collection_pipelines`, `http_routing`, `testing`) are callee-only. The stale note has accumulated across six consecutive milestones without a fix.
- `crates/sdivi-patterns/src/queries/mod.rs:120` — `CALL_DISPATCH` comment lists `P1 > P2=testing > P4=…`, skipping P3 without explanation. A brief note that P3 is reserved for a future category would prevent confusion when a future contributor tries to insert at P2.5.
- `bindings/sdivi-wasm/tests/m23_native.rs` — function name `list_categories_wasm_export_returns_eight_categories` (if still present) is permanently stale vs. the current count of 16. Carry-over from prior milestones; worth renaming in a clean-up pass.
