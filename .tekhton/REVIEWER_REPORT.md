# Reviewer Report — M41: `http_routing` pattern category

**Review cycle:** 1 of 4
**Reviewer:** code-review agent

---

## Verdict
APPROVED_WITH_NOTES

---

## Complex Blockers (senior coder)
None

---

## Simple Blockers (jr coder)
None

---

## Non-Blocking Notes
- `crates/sdivi-patterns/src/queries/mod.rs:127-128` — pre-existing: no blank line between closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. Carry-over from M38/M39/M40; accumulating flagging by successive reviewers.
- `crates/sdivi-patterns/src/queries/mod.rs:33-34` — pre-existing: `category_for_node_kind` doc note says only `logging` is callee-only; `http_routing`, `framework_hooks`, `schema_validation`, `state_store`, and `collection_pipelines` are also callee-only. The list is now five entries stale.
- `bindings/sdivi-wasm/tests/m23_native.rs:48` — pre-existing: test function name `list_categories_wasm_export_returns_eight_categories` is permanently stale (body asserts 15). Carry-over from M34+; will confuse future readers indefinitely until the test function is renamed.

---

## Coverage Gaps
- Next.js App Router route handlers (named exports `GET`, `POST`, `PUT`, etc. from `route.ts` files) are mentioned in the M41 milestone scope but cannot be detected via callee-text on `call_expression` — they use function-export syntax rather than receiver.method patterns. No regex covers them. This is an inherent limitation of the v0 callee-text model; documenting it explicitly in `http_routing.rs` or `docs/pattern-categories.md` would help future readers understand the scope boundary.

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:33` — `category_for_node_kind` doc note listing callee-only categories has been outpaced by five consecutive milestones (M35 `framework_hooks`, M38 `schema_validation`, M39 `state_store`, M40 `collection_pipelines`, M41 `http_routing`). A one-line update would eliminate recurring reviewer notes.
- `crates/sdivi-patterns/src/queries/tests.rs:292-300` — test `null_safety_node_kinds_do_not_match_non_ts_js_languages` remains semantically inverted (name says "do not match" but body asserts `Some("null_safety")` for non-TS/JS languages). Carry-over from M37; still unresolved at M41.
- `crates/sdivi-patterns/src/queries/http_routing.rs` — `PYTHON_RE` (`\.add_url_rule\(`) is receiver-agnostic (any object), while TS/JS and Go regexes use receiver allowlists. Asymmetry is documented in the module doc but represents a style inconsistency across the three language branches of `matches_callee`. Acceptable for v0 given Flask/FastAPI usage patterns; note for any future Python routing expansion.
