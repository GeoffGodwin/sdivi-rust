# Reviewer Report — M39: `state_store` pattern category

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
- `crates/sdivi-core/src/categories.rs:55-59` — `framework_hooks` CATALOG_ENTRIES description still lists `useStore` as an example of the "custom-hook ecosystem". Since M39 routes `useStore` to `state_store` (P5 beats P6), the example is now misleading. The `state_store` description correctly notes the precedence reassignment, but `framework_hooks` should cross-reference it or substitute a non-overlapping example hook.
- `CODER_SUMMARY.md` module placement note says `state_store` is "alphabetical, between `schema_validation` and `state_management`" — actual code correctly places it *after* `state_management` (`state_management` < `state_store` alphabetically). Not a code defect; summary description is inaccurate.

---

## Coverage Gaps
- `bindings/sdivi-wasm/tests/wasm_smoke.rs` name assertions now cover 10 of 13 categories; `resource_management`, `state_management`, and `type_assertions` remain absent from the name-level assertions (pre-existing gap, not introduced by M39 — count assertion prevents silent drift).

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/tests.rs:229` — pre-existing (carried forward from M37/M38): test `null_safety_node_kinds_do_not_match_non_ts_js_languages` has an inverted name (body asserts `Some("null_safety")` for non-TS/JS languages, i.e. a match, not a non-match). Noted in CODER_SUMMARY.md; cleanup deferred.
- `crates/sdivi-patterns/src/queries/mod.rs:84-106` (`category_for_node_kind`) — doc comment cites only `logging` as the callee-only category. `framework_hooks`, `schema_validation`, and `state_store` are now also callee-only (all have empty `NODE_KINDS`); the note should enumerate all such categories to avoid future reader confusion. Pre-existing gap grown by M35/M38/M39.
- `crates/sdivi-patterns/src/queries/mod.rs:119-120` — no blank line between the closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. `CALL_DISPATCH` uses `#[allow(…)]` + `//` (not `///`), so the doc comment attaches correctly to `classify_hint`; however, a blank line would make the separation unambiguous for future editors who might insert an item between the two. Pre-existing from prior milestones.
