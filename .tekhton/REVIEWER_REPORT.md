# Reviewer Report — M23: Pattern Category Contract + WASM `list_categories()`
Review cycle: 1 of 4
Reviewed by: reviewer agent

## Verdict
APPROVED_WITH_NOTES

## Complex Blockers (senior coder)
- None

## Simple Blockers (jr coder)
- None

## Non-Blocking Notes
- `crates/sdivi-patterns/src/queries/mod.rs:279` — Test assertion failure message still reads "logging is catalog-only in v0 for category_for_node_kind". The module-level doc block (lines 24–31) was correctly updated to reflect M33 native classification, but this assertion message wasn't touched. The tested behaviour is still correct (`category_for_node_kind` never returns `Some("logging")`); update the string to "category_for_node_kind never returns logging; callee-text routing via classify_hint" in a future cleanup pass.

## Coverage Gaps
- None

## Drift Observations
- `crates/sdivi-core/src/categories.rs:90-99` — `CATEGORIES` derives from `CATALOG_ENTRIES` by explicit zero-based index (`CATALOG_ENTRIES[0].0` … `CATALOG_ENTRIES[7].0`). If entries are ever reordered the two arrays must be kept in lockstep manually. The existing `list_categories()` doc-test that asserts length and spot-checks names is a sufficient safety net for now; a comment noting the ordering dependency would help the next maintainer.
