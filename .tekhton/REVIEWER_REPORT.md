# Reviewer Report — M38: `schema_validation` pattern category

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
- `bindings/sdivi-wasm/tests/wasm_smoke.rs:245-254` — `list_categories_returns_schema_version_and_expected_count` explicitly checks 9 of 12 category names; `resource_management`, `state_management`, and `type_assertions` are absent from the name assertions. The `len() == 12` count assertion prevents silent drift, but the asymmetry means three categories lack name-level smoke coverage.
- Pre-existing (flagged by coder): `crates/sdivi-patterns/src/queries/tests.rs:194` — `null_safety_node_kinds_do_not_match_non_ts_js_languages` test name is semantically inverted; the body asserts `Some("null_safety")` for non-TS/JS languages. Carry-over from M37, not in scope.
- `crates/sdivi-patterns/src/queries/mod.rs:115-116` — no blank line between the closing `];` of `CALL_DISPATCH` and the `/// Classify…` doc block for `classify_hint`. The CLAUDE.md placement rule is only violated when the inserted item itself carries `///`; `CALL_DISPATCH` uses `#[allow(…)]` + `//`, so `/// Classify…` attaches correctly. A blank line here would remove any ambiguity for future readers.

---

## Coverage Gaps
- `crates/sdivi-patterns/tests/dispatch_disjointness.rs` corpus has no explicit Python `schema_validation` entry. The `corpus_resolves_identically_for_call_node_kind` test mirrors the full corpus over both `call` and `call_expression` node kinds, so the two Python entries in `category_contract_m38.rs` (`Field(default=0)`, `constr(min_length=1)`) are indirectly exercised; but a direct corpus row for Python schema_validation would make slot P4 Python coverage self-documenting in the disjointness suite.
- `wasm_smoke.rs` does not verify `resource_management`, `state_management`, or `type_assertions` by name (pre-existing gap — not introduced by M38).

---

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:84-106` (`category_for_node_kind`) — the function's doc comment cites `logging` as the only callee-only category. `framework_hooks` and `schema_validation` are now also callee-only (empty `NODE_KINDS`); mentioning them alongside `logging` would prevent future contributors from reading the omissions as oversights.
- `crates/sdivi-core/src/categories.rs:77-81` — the `null_safety` description references `fn?.()` via `optional_chain`, but the grammar emits `call_expression` for optional calls. Pre-existing from M37; the code is correct and the test suite is accurate — only the description is misleading.
- Stale comment in `dispatch_disjointness.rs:26` ("At M38, P1/P4/P6/P8/P9 are active") is correctly updated for M38 — previously stale from M35 through M37. Now current.
