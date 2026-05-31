# Reviewer Report — M35: Pattern Category `framework_hooks`
Review cycle: 2 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- Cycle-1 blocker FIXED: `crates/sdivi-core/tests/category_contract.rs` now asserts count=9 at line 132 (`list_categories_returns_exactly_nine_categories`) and includes a new `list_categories_includes_framework_hooks` test at line 142. Blocker resolved.
- Cycle-1 coverage gap FIXED: `bindings/sdivi-wasm/tests/wasm_smoke.rs` line 244 now asserts count=9 and line 250 asserts `names.contains(&"framework_hooks")`. Gap resolved.
- Pre-existing: `docs/pattern-categories.md` KNOWN_OVERLAPS section header still reads "at M34 (P1/P8/P9 active)" — M35 added P6 and this should read "at M35 (P1/P6/P8/P9 active)". Carried forward from cycle 1; minor inconsistency, cosmetic only.
- Pre-existing: stale assertion message at `crates/sdivi-patterns/src/queries/mod.rs:282` ("logging is catalog-only in v0 for category_for_node_kind") — flagged by M23 and M34 reviewers; not introduced by M35.
- Pre-existing: WASM `package.json` version stranded at 0.2.23 — noted by coder; not introduced by M35.

## Coverage Gaps
- None

## Drift Observations
- `crates/sdivi-patterns/src/queries/mod.rs:103` — `CALL_DISPATCH` is a private `const` with only an inline `//` justification comment and no `///` doc block. `sdivi-patterns` lacks `#![deny(missing_docs)]` so this is not a compile error. Worth a documentation sweep when the registry grows past P9.
- `docs/pattern-categories.md` Go corpus — `fmt.Errorf` is classified as `logging` via the `^fmt\.(Print|Println|Printf|Errorf|Fprint|Sprint)` regex. `fmt.Errorf` constructs an error value and does not emit output. Pre-existing M33 inheritance; the eventual Go error-handling pass will need to revisit this regex entry.
